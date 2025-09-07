package main

import (
	"context"
	"fmt"
	"io"
	"log"
	"os"
	"time"

	"github.com/minio/minio-go/v7"
	"github.com/minio/minio-go/v7/pkg/credentials"
	"github.com/nats-io/nats.go"
	"github.com/nats-io/nats.go/jetstream"
)

// NATS to S3 Object Store Bridge
type Bridge struct {
	nc       *nats.Conn
	js       jetstream.JetStream
	s3Client *minio.Client
	bucket   string
}

func NewBridge(natsURL, s3Endpoint, s3Access, s3Secret, bucket string) (*Bridge, error) {
	// Connect to NATS
	nc, err := nats.Connect(natsURL)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to NATS: %w", err)
	}

	// Create JetStream context
	js, err := jetstream.New(nc)
	if err != nil {
		return nil, fmt.Errorf("failed to create JetStream context: %w", err)
	}

	// Initialize MinIO/S3 client
	s3Client, err := minio.New(s3Endpoint, &minio.Options{
		Creds:  credentials.NewStaticV4(s3Access, s3Secret, ""),
		Secure: true,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to create S3 client: %w", err)
	}

	// Ensure bucket exists
	ctx := context.Background()
	exists, err := s3Client.BucketExists(ctx, bucket)
	if err != nil {
		return nil, fmt.Errorf("failed to check bucket: %w", err)
	}
	if !exists {
		err = s3Client.MakeBucket(ctx, bucket, minio.MakeBucketOptions{})
		if err != nil {
			return nil, fmt.Errorf("failed to create bucket: %w", err)
		}
	}

	return &Bridge{
		nc:       nc,
		js:       js,
		s3Client: s3Client,
		bucket:   bucket,
	}, nil
}

// SyncObjectStoreToS3 syncs NATS Object Store to S3
func (b *Bridge) SyncObjectStoreToS3(storeName string) error {
	// Get or create object store
	store, err := b.js.ObjectStore(context.Background(), storeName)
	if err != nil {
		return fmt.Errorf("failed to get object store: %w", err)
	}

	// Watch for changes
	watcher, err := store.Watch()
	if err != nil {
		return fmt.Errorf("failed to create watcher: %w", err)
	}
	defer watcher.Stop()

	log.Printf("Watching NATS Object Store '%s' for changes...", storeName)

	for {
		select {
		case info := <-watcher.Updates():
			if info == nil {
				continue
			}

			// Handle different operations
			if info.Deleted {
				// Delete from S3
				err := b.s3Client.RemoveObject(
					context.Background(),
					b.bucket,
					fmt.Sprintf("%s/%s", storeName, info.Name),
					minio.RemoveObjectOptions{},
				)
				if err != nil {
					log.Printf("Failed to delete %s from S3: %v", info.Name, err)
				} else {
					log.Printf("Deleted %s from S3", info.Name)
				}
			} else {
				// Get object from NATS
				result, err := store.Get(context.Background(), info.Name)
				if err != nil {
					log.Printf("Failed to get %s from NATS: %v", info.Name, err)
					continue
				}

				// Upload to S3
				_, err = b.s3Client.PutObject(
					context.Background(),
					b.bucket,
					fmt.Sprintf("%s/%s", storeName, info.Name),
					result,
					-1,
					minio.PutObjectOptions{
						ContentType: "application/octet-stream",
						UserMetadata: map[string]string{
							"nats-store":    storeName,
							"nats-size":     fmt.Sprintf("%d", info.Size),
							"nats-modified": info.ModTime.Format(time.RFC3339),
						},
					},
				)
				result.Close()

				if err != nil {
					log.Printf("Failed to upload %s to S3: %v", info.Name, err)
				} else {
					log.Printf("Uploaded %s to S3 (size: %d bytes)", info.Name, info.Size)
				}
			}
		}
	}
}

// RestoreFromS3 restores objects from S3 to NATS Object Store
func (b *Bridge) RestoreFromS3(storeName string) error {
	// Create object store
	cfg := jetstream.ObjectStoreConfig{
		Bucket:      storeName,
		Description: "Restored from S3",
	}
	store, err := b.js.CreateObjectStore(context.Background(), cfg)
	if err != nil {
		// Try to get existing store
		store, err = b.js.ObjectStore(context.Background(), storeName)
		if err != nil {
			return fmt.Errorf("failed to create/get object store: %w", err)
		}
	}

	// List objects in S3
	ctx := context.Background()
	prefix := fmt.Sprintf("%s/", storeName)
	
	for object := range b.s3Client.ListObjects(ctx, b.bucket, minio.ListObjectsOptions{
		Prefix:    prefix,
		Recursive: true,
	}) {
		if object.Err != nil {
			return fmt.Errorf("error listing objects: %w", object.Err)
		}

		// Get object from S3
		obj, err := b.s3Client.GetObject(ctx, b.bucket, object.Key, minio.GetObjectOptions{})
		if err != nil {
			log.Printf("Failed to get %s from S3: %v", object.Key, err)
			continue
		}

		// Extract object name (remove prefix)
		objectName := object.Key[len(prefix):]

		// Put to NATS Object Store
		_, err = store.Put(context.Background(), jetstream.ObjectMeta{
			Name: objectName,
		}, obj)
		obj.Close()

		if err != nil {
			log.Printf("Failed to put %s to NATS: %v", objectName, err)
		} else {
			log.Printf("Restored %s to NATS (size: %d bytes)", objectName, object.Size)
		}
	}

	return nil
}

func main() {
	// Configuration from environment
	natsURL := os.Getenv("NATS_URL")
	if natsURL == "" {
		natsURL = "nats://localhost:4222"
	}

	s3Endpoint := os.Getenv("S3_ENDPOINT") // e.g., "s3.wasabisys.com"
	s3Access := os.Getenv("S3_ACCESS_KEY")
	s3Secret := os.Getenv("S3_SECRET_KEY")
	s3Bucket := os.Getenv("S3_BUCKET")

	if s3Endpoint == "" || s3Access == "" || s3Secret == "" || s3Bucket == "" {
		log.Fatal("S3_ENDPOINT, S3_ACCESS_KEY, S3_SECRET_KEY, and S3_BUCKET must be set")
	}

	// Create bridge
	bridge, err := NewBridge(natsURL, s3Endpoint, s3Access, s3Secret, s3Bucket)
	if err != nil {
		log.Fatal(err)
	}
	defer bridge.nc.Close()

	// Example: Sync a specific object store
	storeName := "my-objects"
	
	// Restore from S3 if needed
	if len(os.Args) > 1 && os.Args[1] == "restore" {
		err = bridge.RestoreFromS3(storeName)
		if err != nil {
			log.Fatal(err)
		}
		return
	}

	// Otherwise, sync to S3
	err = bridge.SyncObjectStoreToS3(storeName)
	if err != nil {
		log.Fatal(err)
	}
} 