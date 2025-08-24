# Claude API to NATS Adapter - Implementation Guide

## Technology Stack Recommendations

### Core Implementation: Go
**Rationale**: 
- Excellent NATS client library support
- Strong concurrency model for handling simultaneous conversations
- Efficient memory usage for event processing
- Rich ecosystem for HTTP services and database interactions

### Key Dependencies
```go
// Core NATS integration
"github.com/nats-io/nats.go"
"github.com/nats-io/jetstream"

// HTTP API framework  
"github.com/gin-gonic/gin"
"github.com/gin-contrib/cors"

// Database and persistence
"github.com/lib/pq"              // PostgreSQL driver
"github.com/jmoiron/sqlx"        // SQL extensions
"github.com/golang-migrate/migrate" // Database migrations

// Domain modeling
"github.com/google/uuid"         // UUID generation
"github.com/shopspring/decimal"  // Precise decimals

// Observability
"github.com/prometheus/client_golang"
"go.opentelemetry.io/otel"
"github.com/sirupsen/logrus"

// Testing
"github.com/stretchr/testify"
"github.com/testcontainers/testcontainers-go"
```

## Implementation Architecture

### Project Structure
```
claude-adapter/
├── cmd/
│   └── server/
│       └── main.go              # Application entry point
├── internal/
│   ├── domain/
│   │   ├── conversation.go      # Aggregate root
│   │   ├── events.go           # Domain events  
│   │   ├── values.go           # Value objects
│   │   └── repository.go       # Repository interface
│   ├── infrastructure/
│   │   ├── nats/
│   │   │   ├── adapter.go      # NATS adapter implementation
│   │   │   ├── streams.go      # Stream management
│   │   │   └── consumer.go     # Event consumers
│   │   ├── http/
│   │   │   ├── handlers.go     # HTTP handlers
│   │   │   ├── middleware.go   # CORS, auth, logging
│   │   │   └── router.go       # Route configuration
│   │   ├── persistence/
│   │   │   ├── postgres.go     # PostgreSQL repository
│   │   │   └── migrations/     # Database migrations
│   │   └── claude/
│   │       ├── client.go       # Claude API client
│   │       └── rate_limiter.go # Rate limiting
│   ├── application/
│   │   ├── service.go          # Application services
│   │   ├── handlers.go         # Event handlers
│   │   └── commands.go         # Command handlers
│   └── config/
│       └── config.go           # Configuration management
├── api/
│   └── openapi.yaml            # API specification
├── deployments/
│   ├── docker/
│   │   └── Dockerfile
│   └── kubernetes/
│       ├── deployment.yaml
│       ├── service.yaml
│       └── configmap.yaml
├── tests/
│   ├── integration/
│   ├── unit/
│   └── testdata/
└── scripts/
    ├── setup-nats.sh          # NATS configuration script
    └── migrate.sh             # Database migration script
```

## Implementation Phases

### Phase 1: Core Domain Implementation (Week 1)
```go
// Domain aggregate
type ConversationAggregate struct {
    id               ConversationID
    sessionID        SessionID
    state            ConversationState
    context          ConversationContext
    correlationChains []CorrelationChain
    events           []DomainEvent
}

func (c *ConversationAggregate) SendPrompt(
    prompt Prompt, 
    correlationID CorrelationID,
) error {
    if c.state != ConversationStateActive {
        return ErrInactiveConversation
    }
    
    eventID := EventID(uuid.New())
    event := PromptSentEvent{
        ConversationID: c.id,
        Prompt:         prompt,
        CorrelationID:  correlationID,
        EventID:        eventID,
        OccurredAt:     time.Now(),
    }
    
    c.recordEvent(event)
    c.correlationChains = append(c.correlationChains, 
        NewCorrelationChain(correlationID, eventID))
    
    return nil
}
```

### Phase 2: NATS Infrastructure (Week 2)
```go
// NATS adapter implementation
type NATSAdapter struct {
    nc       *nats.Conn
    js       jetstream.JetStream
    commands jetstream.Consumer
    logger   *logrus.Logger
}

func (a *NATSAdapter) PublishCommand(
    ctx context.Context,
    cmd ClaudeCommand,
) error {
    subject := fmt.Sprintf("claude.cmd.%s.prompt", cmd.SessionID)
    
    data, err := json.Marshal(cmd)
    if err != nil {
        return fmt.Errorf("marshal command: %w", err)
    }
    
    _, err = a.js.Publish(ctx, subject, data)
    if err != nil {
        return fmt.Errorf("publish to NATS: %w", err) 
    }
    
    return nil
}

func (a *NATSAdapter) SubscribeToResponses(
    ctx context.Context,
    handler ResponseHandler,
) error {
    consumer, err := a.js.Consumer(ctx, "CLAUDE_RESPONSES", "claude-resp-distributor")
    if err != nil {
        return fmt.Errorf("get consumer: %w", err)
    }
    
    msgs, err := consumer.Consume(ctx)
    if err != nil {
        return fmt.Errorf("consume messages: %w", err)
    }
    
    go func() {
        for msg := range msgs {
            var response ClaudeResponse
            if err := json.Unmarshal(msg.Data(), &response); err != nil {
                a.logger.WithError(err).Error("unmarshal response")
                msg.Nak()
                continue
            }
            
            if err := handler.Handle(ctx, response); err != nil {
                a.logger.WithError(err).Error("handle response")
                msg.Nak()
                continue  
            }
            
            msg.Ack()
        }
    }()
    
    return nil
}
```

### Phase 3: HTTP API Layer (Week 3)
```go
// HTTP handlers
type ConversationHandler struct {
    service *ConversationService
    logger  *logrus.Logger
}

func (h *ConversationHandler) StartConversation(c *gin.Context) {
    var req StartConversationRequest
    if err := c.ShouldBindJSON(&req); err != nil {
        c.JSON(400, gin.H{"error": "invalid request"})
        return
    }
    
    correlationID := CorrelationID(uuid.New())
    conversation, err := h.service.StartConversation(
        c.Request.Context(),
        req.InitialPrompt,
        correlationID,
    )
    if err != nil {
        h.logger.WithError(err).Error("start conversation")
        c.JSON(500, gin.H{"error": "internal server error"})
        return
    }
    
    c.JSON(201, gin.H{
        "conversation_id": conversation.ID,
        "session_id":      conversation.SessionID,
        "correlation_id":  correlationID,
    })
}

func (h *ConversationHandler) SendPrompt(c *gin.Context) {
    conversationID := ConversationID(c.Param("id"))
    
    var req SendPromptRequest
    if err := c.ShouldBindJSON(&req); err != nil {
        c.JSON(400, gin.H{"error": "invalid request"})
        return
    }
    
    correlationID := CorrelationID(uuid.New())
    err := h.service.SendPrompt(
        c.Request.Context(),
        conversationID,
        req.Prompt,
        correlationID,
    )
    if err != nil {
        h.logger.WithError(err).Error("send prompt")
        c.JSON(500, gin.H{"error": "internal server error"})
        return
    }
    
    c.JSON(202, gin.H{
        "correlation_id": correlationID,
        "status": "processing",
    })
}
```

### Phase 4: Claude API Integration (Week 4)
```go
// Claude API client
type ClaudeClient struct {
    httpClient *http.Client
    apiKey     string
    baseURL    string
    rateLimiter *rate.Limiter
}

func (c *ClaudeClient) SendPrompt(
    ctx context.Context,
    prompt Prompt,
    context ConversationContext,
) (*ClaudeResponse, error) {
    // Rate limiting
    if err := c.rateLimiter.Wait(ctx); err != nil {
        return nil, fmt.Errorf("rate limit: %w", err)
    }
    
    reqBody := ClaudeAPIRequest{
        Model:     "claude-3-sonnet-20240229",
        Messages:  buildMessages(prompt, context),
        MaxTokens: 4000,
    }
    
    body, err := json.Marshal(reqBody)
    if err != nil {
        return nil, fmt.Errorf("marshal request: %w", err)
    }
    
    req, err := http.NewRequestWithContext(
        ctx, "POST", c.baseURL+"/v1/messages", bytes.NewReader(body))
    if err != nil {
        return nil, fmt.Errorf("create request: %w", err)
    }
    
    req.Header.Set("Authorization", "Bearer "+c.apiKey)
    req.Header.Set("Content-Type", "application/json")
    req.Header.Set("anthropic-version", "2023-06-01")
    
    resp, err := c.httpClient.Do(req)
    if err != nil {
        return nil, fmt.Errorf("http request: %w", err)
    }
    defer resp.Body.Close()
    
    if resp.StatusCode != 200 {
        return nil, fmt.Errorf("Claude API error: %d", resp.StatusCode)
    }
    
    var apiResp ClaudeAPIResponse
    if err := json.NewDecoder(resp.Body).Decode(&apiResp); err != nil {
        return nil, fmt.Errorf("decode response: %w", err)
    }
    
    return &ClaudeResponse{
        Content:      apiResp.Content[0].Text,
        Usage:        apiResp.Usage,
        FinishReason: apiResp.StopReason,
    }, nil
}
```

## Deployment Strategy

### Docker Configuration
```dockerfile
FROM golang:1.21-alpine AS builder

WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download

COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -o claude-adapter cmd/server/main.go

FROM alpine:latest
RUN apk --no-cache add ca-certificates tzdata
WORKDIR /root/

COPY --from=builder /app/claude-adapter .
COPY --from=builder /app/migrations ./migrations

CMD ["./claude-adapter"]
```

### Kubernetes Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: claude-adapter
  namespace: claude-adapter
spec:
  replicas: 2
  selector:
    matchLabels:
      app: claude-adapter
  template:
    metadata:
      labels:
        app: claude-adapter
    spec:
      containers:
      - name: claude-adapter
        image: claude-adapter:latest
        ports:
        - containerPort: 8080
        env:
        - name: NATS_URL
          value: "nats://nats:4222"
        - name: CLAUDE_API_KEY
          valueFrom:
            secretKeyRef:
              name: claude-secrets
              key: api-key
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: postgres-secrets  
              key: connection-string
        volumeMounts:
        - name: nats-creds
          mountPath: /etc/nats
          readOnly: true
      volumes:
      - name: nats-creds
        secret:
          secretName: nats-credentials
```

## Testing Strategy

### Integration Tests
```go
func TestConversationLifecycle(t *testing.T) {
    // Setup test containers
    natsContainer := setupNATSContainer(t)
    defer natsContainer.Terminate(context.Background())
    
    postgresContainer := setupPostgresContainer(t)
    defer postgresContainer.Terminate(context.Background())
    
    // Initialize adapter with test configuration
    adapter := NewClaudeAdapter(Config{
        NATSUrl:     natsContainer.ConnectionString(),
        DatabaseUrl: postgresContainer.ConnectionString(),
    })
    
    // Test complete conversation flow
    t.Run("StartConversation", func(t *testing.T) {
        req := StartConversationRequest{
            InitialPrompt: "Hello, Claude!",
        }
        
        resp, err := adapter.StartConversation(context.Background(), req)
        require.NoError(t, err)
        assert.NotEmpty(t, resp.ConversationID)
        assert.NotEmpty(t, resp.SessionID)
    })
    
    t.Run("SendPromptAndReceiveResponse", func(t *testing.T) {
        // Mock Claude API response
        mockResponse := &ClaudeResponse{
            Content: "Hello! How can I help you?",
            Usage: Usage{
                PromptTokens:     10,
                CompletionTokens: 15,
            },
        }
        
        // Send prompt
        err := adapter.SendPrompt(context.Background(), conversationID, "Test prompt")
        require.NoError(t, err)
        
        // Verify response received
        response := waitForResponse(t, adapter, correlationID)
        assert.Equal(t, mockResponse.Content, response.Content)
    })
}
```

## Monitoring and Observability

### Metrics Collection
```go
var (
    requestsTotal = prometheus.NewCounterVec(
        prometheus.CounterOpts{
            Name: "claude_requests_total",
            Help: "Total number of Claude API requests",
        },
        []string{"method", "status"},
    )
    
    responseDuration = prometheus.NewHistogramVec(
        prometheus.HistogramOpts{
            Name: "claude_response_duration_seconds", 
            Help: "Claude API response duration",
        },
        []string{"conversation_id"},
    )
    
    activeConversations = prometheus.NewGauge(
        prometheus.GaugeOpts{
            Name: "conversations_active_total",
            Help: "Number of active conversations",
        },
    )
)

func (s *ConversationService) SendPrompt(
    ctx context.Context,
    conversationID ConversationID,
    prompt Prompt,
) error {
    timer := prometheus.NewTimer(responseDuration.WithLabelValues(
        conversationID.String()))
    defer timer.ObserveDuration()
    
    requestsTotal.WithLabelValues("send_prompt", "attempted").Inc()
    
    err := s.doSendPrompt(ctx, conversationID, prompt)
    if err != nil {
        requestsTotal.WithLabelValues("send_prompt", "error").Inc()
        return err
    }
    
    requestsTotal.WithLabelValues("send_prompt", "success").Inc()
    return nil
}
```

## Security Considerations

### Authentication & Authorization
```go
// JWT middleware for API authentication
func JWTAuthMiddleware(secret string) gin.HandlerFunc {
    return gin.HandlerFunc(func(c *gin.Context) {
        token := c.GetHeader("Authorization")
        if token == "" {
            c.AbortWithStatusJSON(401, gin.H{"error": "missing token"})
            return
        }
        
        // Validate JWT token
        claims, err := validateJWT(token, secret)
        if err != nil {
            c.AbortWithStatusJSON(401, gin.H{"error": "invalid token"})
            return
        }
        
        // Set user context
        c.Set("user_id", claims.UserID)
        c.Set("permissions", claims.Permissions)
        c.Next()
    })
}

// Rate limiting per user
func UserRateLimitMiddleware() gin.HandlerFunc {
    limiters := make(map[string]*rate.Limiter)
    mu := sync.RWMutex{}
    
    return gin.HandlerFunc(func(c *gin.Context) {
        userID := c.GetString("user_id")
        
        mu.Lock()
        limiter, exists := limiters[userID]
        if !exists {
            limiter = rate.NewLimiter(rate.Limit(10), 10) // 10 req/sec burst 10
            limiters[userID] = limiter
        }
        mu.Unlock()
        
        if !limiter.Allow() {
            c.AbortWithStatusJSON(429, gin.H{"error": "rate limit exceeded"})
            return
        }
        
        c.Next()
    })
}
```

### NATS Security
```bash
# Generate NATS credentials using NSC
nsc add account CLAUDE_ADAPTER
nsc add user -a CLAUDE_ADAPTER claude-adapter-service

# Set permissions
nsc edit user claude-adapter-service \
    --allow-pub "claude.event.>,claude.resp.>" \
    --allow-sub "claude.cmd.>" \
    --max-payload 1MB

# Generate credentials file
nsc generate creds -a CLAUDE_ADAPTER -n claude-adapter-service > claude-adapter.creds
```

## Performance Optimization

### Connection Pooling
```go
// Database connection pool
func NewPostgresRepository(databaseURL string) (*PostgresRepository, error) {
    db, err := sqlx.Connect("postgres", databaseURL)
    if err != nil {
        return nil, fmt.Errorf("connect to database: %w", err)
    }
    
    // Configure connection pool
    db.SetMaxOpenConns(25)
    db.SetMaxIdleConns(5)
    db.SetConnMaxLifetime(5 * time.Minute)
    
    return &PostgresRepository{db: db}, nil
}

// NATS connection with options
func NewNATSConnection(url string) (*nats.Conn, error) {
    opts := []nats.Option{
        nats.MaxReconnects(10),
        nats.ReconnectWait(2 * time.Second),
        nats.DisconnectErrHandler(func(nc *nats.Conn, err error) {
            log.Printf("NATS disconnected: %v", err)
        }),
        nats.ReconnectHandler(func(nc *nats.Conn) {
            log.Printf("NATS reconnected")
        }),
    }
    
    return nats.Connect(url, opts...)
}
```

## Next Steps and Success Criteria

### Implementation Milestones
1. **Week 1**: Domain model and core business logic
2. **Week 2**: NATS infrastructure integration
3. **Week 3**: HTTP API and request handling  
4. **Week 4**: Claude API client and response processing
5. **Week 5**: Testing, monitoring, and deployment
6. **Week 6**: Performance optimization and security hardening

### Success Criteria
- [ ] HTTP requests are successfully transformed into NATS events
- [ ] Claude API responses are properly correlated with original requests
- [ ] Conversation state is maintained across multiple interactions
- [ ] Rate limiting prevents API abuse
- [ ] Error conditions are handled gracefully
- [ ] System can handle 100 concurrent conversations
- [ ] Event sourcing provides complete audit trail
- [ ] Monitoring provides visibility into system health

### Production Readiness Checklist
- [ ] Comprehensive test coverage (>80%)
- [ ] Load testing validates performance requirements
- [ ] Security audit completed
- [ ] Monitoring and alerting configured
- [ ] Documentation complete
- [ ] Disaster recovery procedures defined
- [ ] Runbook for operational procedures

This implementation guide provides a complete roadmap for building the Claude API to NATS adapter following CIM architectural principles while maintaining clean boundaries and proper integration patterns.