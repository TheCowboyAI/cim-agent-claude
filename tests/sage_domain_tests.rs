//! SAGE Domain Model Unit Tests
//!
//! Tests for SAGE domain model functionality including aggregates, events, 
//! state machines, and commands WITHOUT requiring NATS infrastructure.
//! These tests validate the mathematical foundations and domain logic.

use std::collections::HashMap;
use chrono::Utc;

// Import SAGE domain model components from the subagents module
use cim_agent_claude::subagents::sage::{
    SageOrchestratorAggregate, SageSessionId, SageSessionState, SageSessionMetadata,
    SageEvent, SageCommand, SageSessionStateMachine, SageError
};

/// Test SAGE Session ID strong typing
#[cfg(test)]
mod sage_session_id_tests {
    use super::*;

    #[test]
    fn test_new_session_id_is_unique() {
        // Given/When
        let id1 = SageSessionId::new();
        let id2 = SageSessionId::new();
        
        // Then
        assert_ne!(id1, id2);
        assert!(!id1.as_str().is_empty());
        assert!(!id2.as_str().is_empty());
    }

    #[test]
    fn test_session_id_from_string() {
        // Given
        let test_id = "test-session-123";
        
        // When
        let session_id = SageSessionId::from_string(test_id.to_string());
        
        // Then
        assert_eq!(session_id.as_str(), test_id);
    }

    #[test]
    fn test_session_id_display() {
        // Given
        let test_id = "display-test-456";
        let session_id = SageSessionId::from_string(test_id.to_string());
        
        // When
        let displayed = format!("{}", session_id);
        
        // Then
        assert_eq!(displayed, test_id);
    }
}

/// Test SAGE Session State Machine
#[cfg(test)]
mod sage_state_machine_tests {
    use super::*;

    #[test]
    fn test_valid_state_transitions() {
        use SageSessionState::*;
        
        // Valid transitions
        assert!(SageSessionStateMachine::can_transition(&Created, &AnalyzingQuery));
        assert!(SageSessionStateMachine::can_transition(&AnalyzingQuery, &RoutingToExperts));
        assert!(SageSessionStateMachine::can_transition(&RoutingToExperts, &ProcessingWithExperts));
        assert!(SageSessionStateMachine::can_transition(&ProcessingWithExperts, &SynthesizingResponse));
        assert!(SageSessionStateMachine::can_transition(&SynthesizingResponse, &Completed));
        
        // Can transition to Failed from any state
        assert!(SageSessionStateMachine::can_transition(&Created, &Failed { error: "test".to_string() }));
        assert!(SageSessionStateMachine::can_transition(&AnalyzingQuery, &Failed { error: "test".to_string() }));
        assert!(SageSessionStateMachine::can_transition(&ProcessingWithExperts, &Failed { error: "test".to_string() }));
        
        // Can transition to Cancelled from any state
        assert!(SageSessionStateMachine::can_transition(&Created, &Cancelled));
        assert!(SageSessionStateMachine::can_transition(&ProcessingWithExperts, &Cancelled));
        assert!(SageSessionStateMachine::can_transition(&SynthesizingResponse, &Cancelled));
    }

    #[test]
    fn test_invalid_state_transitions() {
        use SageSessionState::*;
        
        // Invalid transitions
        assert!(!SageSessionStateMachine::can_transition(&Created, &ProcessingWithExperts));
        assert!(!SageSessionStateMachine::can_transition(&AnalyzingQuery, &SynthesizingResponse));
        assert!(!SageSessionStateMachine::can_transition(&Completed, &Created));
        assert!(!SageSessionStateMachine::can_transition(&Completed, &AnalyzingQuery));
        assert!(!SageSessionStateMachine::can_transition(&Failed { error: "test".to_string() }, &AnalyzingQuery));
    }

    #[test]
    fn test_state_transition_execution() {
        use SageSessionState::*;
        
        // Valid transition
        let from = Created;
        let to = AnalyzingQuery;
        let result = SageSessionStateMachine::transition(from, to.clone());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to);
    }

    #[test]
    fn test_state_transition_validation_error() {
        use SageSessionState::*;
        
        // Invalid transition
        let from = Completed;
        let to = Created;
        let result = SageSessionStateMachine::transition(from, to);
        
        assert!(result.is_err());
    }
}

/// Test SAGE Orchestrator Aggregate
#[cfg(test)]
mod sage_aggregate_tests {
    use super::*;

    #[test]
    fn test_new_sage_aggregate_creation() {
        // Given
        let session_id = SageSessionId::new();
        
        // When
        let aggregate = SageOrchestratorAggregate::new(session_id.clone());
        
        // Then
        assert_eq!(aggregate.aggregate_id(), &session_id);
        assert_eq!(aggregate.state, SageSessionState::Created);
        assert_eq!(aggregate.version(), 0);
        assert!(aggregate.events().is_empty());
        assert!(aggregate.metadata.user_query.is_empty());
    }

    #[test]
    fn test_session_created_event_application() {
        // Given
        let session_id = SageSessionId::new();
        let mut aggregate = SageOrchestratorAggregate::new(session_id.clone());
        
        let metadata = SageSessionMetadata {
            user_query: "Test query for CIM creation".to_string(),
            selected_expert: Some("cim-expert".to_string()),
            session_context: HashMap::new(),
            created_at: Utc::now(),
            last_updated: Utc::now(),
        };
        
        let event = SageEvent::SessionCreated {
            session_id: session_id.clone(),
            metadata: metadata.clone(),
            timestamp: Utc::now(),
        };
        
        // When
        let result = aggregate.apply_event(event);
        
        // Then
        assert!(result.is_ok());
        assert_eq!(aggregate.state, SageSessionState::Created);
        assert_eq!(aggregate.version(), 1);
        assert_eq!(aggregate.events().len(), 1);
        assert_eq!(aggregate.metadata.user_query, "Test query for CIM creation");
        assert_eq!(aggregate.metadata.selected_expert, Some("cim-expert".to_string()));
    }

    #[test]
    fn test_query_received_event_progression() {
        // Given
        let session_id = SageSessionId::new();
        let mut aggregate = SageOrchestratorAggregate::new(session_id.clone());
        
        // First apply SessionCreated to get to Created state
        let metadata = SageSessionMetadata {
            user_query: "Test query".to_string(),
            selected_expert: None,
            session_context: HashMap::new(),
            created_at: Utc::now(),
            last_updated: Utc::now(),
        };
        
        let session_created = SageEvent::SessionCreated {
            session_id: session_id.clone(),
            metadata,
            timestamp: Utc::now(),
        };
        aggregate.apply_event(session_created).unwrap();
        
        let query_event = SageEvent::QueryReceived {
            session_id: session_id.clone(),
            query: "How do I build a CIM for e-commerce?".to_string(),
            timestamp: Utc::now(),
        };
        
        // When
        let result = aggregate.apply_event(query_event);
        
        // Then
        assert!(result.is_ok());
        assert_eq!(aggregate.state, SageSessionState::AnalyzingQuery);
        assert_eq!(aggregate.version(), 2);
        assert_eq!(aggregate.events().len(), 2);
    }

    #[test]
    fn test_complete_orchestration_workflow() {
        // Given
        let session_id = SageSessionId::new();
        let mut aggregate = SageOrchestratorAggregate::new(session_id.clone());
        
        let events = vec![
            SageEvent::SessionCreated {
                session_id: session_id.clone(),
                metadata: SageSessionMetadata {
                    user_query: "Build healthcare CIM system".to_string(),
                    selected_expert: None,
                    session_context: HashMap::new(),
                    created_at: Utc::now(),
                    last_updated: Utc::now(),
                },
                timestamp: Utc::now(),
            },
            SageEvent::QueryReceived {
                session_id: session_id.clone(),
                query: "Build healthcare CIM system".to_string(),
                timestamp: Utc::now(),
            },
            SageEvent::QueryAnalyzed {
                session_id: session_id.clone(),
                routing_strategy: "multi_expert_coordination".to_string(),
                selected_experts: vec!["cim-expert".to_string(), "ddd-expert".to_string(), "nats-expert".to_string()],
                confidence_score: 0.95,
                timestamp: Utc::now(),
            },
            SageEvent::RoutedToExperts {
                session_id: session_id.clone(),
                experts: vec!["cim-expert".to_string(), "ddd-expert".to_string(), "nats-expert".to_string()],
                timestamp: Utc::now(),
            },
            SageEvent::ExpertResponseReceived {
                session_id: session_id.clone(),
                expert_id: "cim-expert".to_string(),
                response: "Healthcare CIM requires patient data categories...".to_string(),
                confidence_score: 0.9,
                timestamp: Utc::now(),
            },
            SageEvent::ResponseSynthesized {
                session_id: session_id.clone(),
                synthesized_response: "Complete healthcare CIM architecture with domain boundaries...".to_string(),
                contributing_experts: vec!["cim-expert".to_string(), "ddd-expert".to_string(), "nats-expert".to_string()],
                timestamp: Utc::now(),
            },
            SageEvent::SessionCompleted {
                session_id: session_id.clone(),
                duration_ms: 5000,
                timestamp: Utc::now(),
            },
        ];
        
        // When - Apply all events in sequence
        for event in events {
            let result = aggregate.apply_event(event);
            assert!(result.is_ok(), "Event application should succeed");
        }
        
        // Then
        assert_eq!(aggregate.state, SageSessionState::Completed);
        assert_eq!(aggregate.version(), 7);
        assert_eq!(aggregate.events().len(), 7);
    }

    #[test]
    fn test_session_failure_handling() {
        // Given
        let session_id = SageSessionId::new();
        let mut aggregate = SageOrchestratorAggregate::new(session_id.clone());
        
        // Apply initial events
        let metadata = SageSessionMetadata {
            user_query: "Invalid query".to_string(),
            selected_expert: None,
            session_context: HashMap::new(),
            created_at: Utc::now(),
            last_updated: Utc::now(),
        };
        
        aggregate.apply_event(SageEvent::SessionCreated {
            session_id: session_id.clone(),
            metadata,
            timestamp: Utc::now(),
        }).unwrap();
        
        aggregate.apply_event(SageEvent::QueryReceived {
            session_id: session_id.clone(),
            query: "Invalid query".to_string(),
            timestamp: Utc::now(),
        }).unwrap();
        
        let failure_event = SageEvent::SessionFailed {
            session_id: session_id.clone(),
            error: "Unable to parse query requirements".to_string(),
            timestamp: Utc::now(),
        };
        
        // When
        let result = aggregate.apply_event(failure_event);
        
        // Then
        assert!(result.is_ok());
        assert!(matches!(aggregate.state, SageSessionState::Failed { .. }));
        if let SageSessionState::Failed { error } = &aggregate.state {
            assert_eq!(error, "Unable to parse query requirements");
        }
        assert_eq!(aggregate.version(), 3);
    }

    #[test]
    fn test_event_session_id_validation() {
        // Given
        let session_id1 = SageSessionId::new();
        let session_id2 = SageSessionId::new(); // Different session ID
        let mut aggregate = SageOrchestratorAggregate::new(session_id1.clone());
        
        let wrong_session_event = SageEvent::QueryReceived {
            session_id: session_id2, // Wrong session ID
            query: "Test".to_string(),
            timestamp: Utc::now(),
        };
        
        // When
        let result = aggregate.apply_event(wrong_session_event);
        
        // Then
        assert!(result.is_err());
        assert_eq!(aggregate.version(), 0); // No change to aggregate state
    }
}

/// Test SAGE Command Handling
#[cfg(test)]
mod sage_command_handling_tests {
    use super::*;

    #[test]
    fn test_start_session_command() {
        // Given
        let session_id = SageSessionId::new();
        let mut aggregate = SageOrchestratorAggregate::new(session_id.clone());
        
        let command = SageCommand::StartSession {
            user_query: "Create a CIM for inventory management".to_string(),
            context: {
                let mut ctx = HashMap::new();
                ctx.insert("domain".to_string(), "inventory".to_string());
                ctx
            },
            selected_expert: Some("ddd-expert".to_string()),
        };
        
        // When
        let result = aggregate.handle_command(command);
        
        // Then
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 2); // SessionCreated + QueryReceived
        
        // Verify first event is SessionCreated
        match &events[0] {
            SageEvent::SessionCreated { session_id: evt_id, metadata, .. } => {
                assert_eq!(evt_id, &session_id);
                assert_eq!(metadata.user_query, "Create a CIM for inventory management");
                assert_eq!(metadata.selected_expert, Some("ddd-expert".to_string()));
                assert_eq!(metadata.session_context.get("domain"), Some(&"inventory".to_string()));
            }
            _ => panic!("Expected SessionCreated event"),
        }
        
        // Verify second event is QueryReceived
        match &events[1] {
            SageEvent::QueryReceived { session_id: evt_id, query, .. } => {
                assert_eq!(evt_id, &session_id);
                assert_eq!(query, "Create a CIM for inventory management");
            }
            _ => panic!("Expected QueryReceived event"),
        }
    }

    #[test]
    fn test_route_to_experts_command_valid_state() {
        // Given
        let session_id = SageSessionId::new();
        let mut aggregate = SageOrchestratorAggregate::new(session_id.clone());
        
        // Set up aggregate in correct state for routing
        aggregate.state = SageSessionState::RoutingToExperts;
        
        let command = SageCommand::RouteToExperts {
            session_id: session_id.clone(),
            experts: vec!["cim-expert".to_string(), "nats-expert".to_string()],
        };
        
        // When
        let result = aggregate.handle_command(command);
        
        // Then
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            SageEvent::RoutedToExperts { session_id: evt_id, experts, .. } => {
                assert_eq!(evt_id, &session_id);
                assert_eq!(experts.len(), 2);
                assert!(experts.contains(&"cim-expert".to_string()));
                assert!(experts.contains(&"nats-expert".to_string()));
            }
            _ => panic!("Expected RoutedToExperts event"),
        }
    }

    #[test]
    fn test_route_to_experts_command_invalid_state() {
        // Given
        let session_id = SageSessionId::new();
        let mut aggregate = SageOrchestratorAggregate::new(session_id.clone());
        
        // State is Created, not RoutingToExperts
        assert_eq!(aggregate.state, SageSessionState::Created);
        
        let command = SageCommand::RouteToExperts {
            session_id: session_id.clone(),
            experts: vec!["cim-expert".to_string()],
        };
        
        // When
        let result = aggregate.handle_command(command);
        
        // Then
        assert!(result.is_err());
        // Should get InvalidOperation error
    }

    #[test]
    fn test_synthesize_response_command() {
        // Given
        let session_id = SageSessionId::new();
        let mut aggregate = SageOrchestratorAggregate::new(session_id.clone());
        
        // Set up aggregate in correct state
        aggregate.state = SageSessionState::ProcessingWithExperts;
        
        let expert_responses = vec![
            ("cim-expert".to_string(), "CIM architecture should use Category Theory...".to_string(), 0.9),
            ("ddd-expert".to_string(), "Domain boundaries for healthcare: Patient, Treatment...".to_string(), 0.85),
            ("nats-expert".to_string(), "NATS subjects: health.patient.>, health.treatment.>...".to_string(), 0.8),
        ];
        
        let command = SageCommand::SynthesizeResponse {
            session_id: session_id.clone(),
            expert_responses,
        };
        
        // When
        let result = aggregate.handle_command(command);
        
        // Then
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            SageEvent::ResponseSynthesized { session_id: evt_id, synthesized_response, contributing_experts, .. } => {
                assert_eq!(evt_id, &session_id);
                assert!(synthesized_response.contains("CIM architecture should use Category Theory"));
                assert!(synthesized_response.contains("Domain boundaries for healthcare"));
                assert!(synthesized_response.contains("NATS subjects"));
                assert_eq!(contributing_experts.len(), 3);
                assert!(contributing_experts.contains(&"cim-expert".to_string()));
                assert!(contributing_experts.contains(&"ddd-expert".to_string()));
                assert!(contributing_experts.contains(&"nats-expert".to_string()));
            }
            _ => panic!("Expected ResponseSynthesized event"),
        }
    }

    #[test]
    fn test_complete_session_command() {
        // Given
        let session_id = SageSessionId::new();
        let mut aggregate = SageOrchestratorAggregate::new(session_id.clone());
        
        // Set up aggregate in correct state
        aggregate.state = SageSessionState::SynthesizingResponse;
        
        let command = SageCommand::CompleteSession {
            session_id: session_id.clone(),
        };
        
        // When
        let result = aggregate.handle_command(command);
        
        // Then
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            SageEvent::SessionCompleted { session_id: evt_id, duration_ms, .. } => {
                assert_eq!(evt_id, &session_id);
                assert_eq!(*duration_ms, 0); // Placeholder value in current implementation
            }
            _ => panic!("Expected SessionCompleted event"),
        }
    }

    #[test]
    fn test_command_session_id_validation() {
        // Given
        let session_id1 = SageSessionId::new();
        let session_id2 = SageSessionId::new(); // Different session ID
        let mut aggregate = SageOrchestratorAggregate::new(session_id1.clone());
        
        // Set up aggregate in correct state but use wrong session ID in command
        aggregate.state = SageSessionState::RoutingToExperts;
        
        let command = SageCommand::RouteToExperts {
            session_id: session_id2, // Wrong session ID
            experts: vec!["cim-expert".to_string()],
        };
        
        // When
        let result = aggregate.handle_command(command);
        
        // Then
        assert!(result.is_err());
        // Should get BusinessRuleViolation error about session ID mismatch
    }
}

/// Test SAGE Event Domain Event Interface
#[cfg(test)]
mod sage_event_domain_tests {
    use super::*;
    use cim_domain::DomainEvent;

    #[test]
    fn test_sage_event_types() {
        let session_id = SageSessionId::new();
        
        let events = vec![
            SageEvent::SessionCreated {
                session_id: session_id.clone(),
                metadata: SageSessionMetadata {
                    user_query: "test".to_string(),
                    selected_expert: None,
                    session_context: HashMap::new(),
                    created_at: Utc::now(),
                    last_updated: Utc::now(),
                },
                timestamp: Utc::now(),
            },
            SageEvent::QueryReceived {
                session_id: session_id.clone(),
                query: "test query".to_string(),
                timestamp: Utc::now(),
            },
            SageEvent::SessionCompleted {
                session_id: session_id.clone(),
                duration_ms: 1000,
                timestamp: Utc::now(),
            },
        ];
        
        // Test event types
        assert_eq!(events[0].event_type(), "sage.session.created");
        assert_eq!(events[1].event_type(), "sage.query.received");
        assert_eq!(events[2].event_type(), "sage.session.completed");
        
        // Test subjects
        for event in &events {
            let subject = event.subject();
            assert!(subject.starts_with("cim.events.sage."));
            assert!(subject.contains(session_id.as_str()));
        }
        
        // Test aggregate IDs
        for event in &events {
            let aggregate_id = event.aggregate_id();
            // All events should have the same aggregate ID (the session ID)
            assert_eq!(aggregate_id.to_string(), session_id.as_str());
        }
    }

    #[test]
    fn test_event_subject_patterns() {
        let session_id = SageSessionId::new();
        
        let event = SageEvent::QueryAnalyzed {
            session_id: session_id.clone(),
            routing_strategy: "single_expert".to_string(),
            selected_experts: vec!["cim-expert".to_string()],
            confidence_score: 0.9,
            timestamp: Utc::now(),
        };
        
        let subject = event.subject();
        assert_eq!(subject, format!("cim.events.sage.{}", session_id));
        assert_eq!(event.event_type(), "sage.query.analyzed");
    }
}

/// Integration test for complete SAGE workflow without NATS
#[cfg(test)]
mod sage_workflow_integration_tests {
    use super::*;

    #[test]
    fn test_complete_cim_creation_workflow() {
        // Given: A complete CIM creation session
        let session_id = SageSessionId::new();
        let mut aggregate = SageOrchestratorAggregate::new(session_id.clone());
        
        // When: Execute complete workflow through commands and events
        
        // 1. Start session
        let start_command = SageCommand::StartSession {
            user_query: "Create a complete CIM system for mortgage lending with NATS infrastructure, domain boundaries, and GUI interface".to_string(),
            context: {
                let mut ctx = HashMap::new();
                ctx.insert("domain".to_string(), "financial_services".to_string());
                ctx.insert("complexity".to_string(), "high".to_string());
                ctx
            },
            selected_expert: None, // Let SAGE decide
        };
        
        let start_events = aggregate.handle_command(start_command).unwrap();
        assert_eq!(start_events.len(), 2);
        
        for event in start_events {
            aggregate.apply_event(event).unwrap();
        }
        assert_eq!(aggregate.state, SageSessionState::AnalyzingQuery);
        
        // 2. Query analysis (simulate SAGE analysis)
        let analysis_event = SageEvent::QueryAnalyzed {
            session_id: session_id.clone(),
            routing_strategy: "multi_expert_parallel_coordination".to_string(),
            selected_experts: vec![
                "cim-expert".to_string(),
                "ddd-expert".to_string(), 
                "event-storming-expert".to_string(),
                "nats-expert".to_string(),
                "iced-ui-expert".to_string(),
            ],
            confidence_score: 0.95,
            timestamp: Utc::now(),
        };
        
        aggregate.apply_event(analysis_event).unwrap();
        assert_eq!(aggregate.state, SageSessionState::RoutingToExperts);
        
        // 3. Route to experts
        let routing_command = SageCommand::RouteToExperts {
            session_id: session_id.clone(),
            experts: vec![
                "cim-expert".to_string(),
                "ddd-expert".to_string(),
                "nats-expert".to_string(),
            ],
        };
        
        let routing_events = aggregate.handle_command(routing_command).unwrap();
        for event in routing_events {
            aggregate.apply_event(event).unwrap();
        }
        assert_eq!(aggregate.state, SageSessionState::ProcessingWithExperts);
        
        // 4. Receive expert responses (simulate expert coordination)
        let expert_responses = vec![
            ("cim-expert", "Mortgage lending CIM requires Category Theory foundations with Loan, Borrower, Property entities. Use IPLD for document content-addressing and ensure immutable audit trails."),
            ("ddd-expert", "Domain boundaries: Loan Origination (Application, Underwriting), Loan Servicing (Payment, Escrow), Property Valuation (Appraisal, Inspection). Each aggregate maintains invariants."),
            ("nats-expert", "NATS subjects: mortgage.loan.>, mortgage.borrower.>, mortgage.property.>. Use JetStream for compliance audit trails and KV store for MRU loan lists."),
        ];
        
        for (expert_id, response) in &expert_responses {
            let expert_response_event = SageEvent::ExpertResponseReceived {
                session_id: session_id.clone(),
                expert_id: expert_id.to_string(),
                response: response.to_string(),
                confidence_score: 0.88,
                timestamp: Utc::now(),
            };
            aggregate.apply_event(expert_response_event).unwrap();
        }
        
        // 5. Synthesize response
        let synthesis_command = SageCommand::SynthesizeResponse {
            session_id: session_id.clone(),
            expert_responses: expert_responses.into_iter()
                .map(|(id, resp)| (id.to_string(), resp.to_string(), 0.88))
                .collect(),
        };
        
        let synthesis_events = aggregate.handle_command(synthesis_command).unwrap();
        for event in synthesis_events {
            aggregate.apply_event(event).unwrap();
        }
        assert_eq!(aggregate.state, SageSessionState::SynthesizingResponse);
        
        // 6. Complete session
        let complete_command = SageCommand::CompleteSession {
            session_id: session_id.clone(),
        };
        
        let complete_events = aggregate.handle_command(complete_command).unwrap();
        for event in complete_events {
            aggregate.apply_event(event).unwrap();
        }
        
        // Then: Verify complete workflow
        assert_eq!(aggregate.state, SageSessionState::Completed);
        assert!(aggregate.version() > 7); // Multiple events applied
        
        // Verify the session has comprehensive orchestration history
        let events = aggregate.events();
        assert!(events.iter().any(|e| matches!(e, SageEvent::SessionCreated { .. })));
        assert!(events.iter().any(|e| matches!(e, SageEvent::QueryReceived { .. })));
        assert!(events.iter().any(|e| matches!(e, SageEvent::QueryAnalyzed { .. })));
        assert!(events.iter().any(|e| matches!(e, SageEvent::RoutedToExperts { .. })));
        assert!(events.iter().any(|e| matches!(e, SageEvent::ExpertResponseReceived { .. })));
        assert!(events.iter().any(|e| matches!(e, SageEvent::ResponseSynthesized { .. })));
        assert!(events.iter().any(|e| matches!(e, SageEvent::SessionCompleted { .. })));
        
        // Verify metadata contains the original complex query
        assert!(aggregate.metadata.user_query.contains("mortgage lending"));
        assert!(aggregate.metadata.user_query.contains("NATS infrastructure"));
        assert!(aggregate.metadata.user_query.contains("domain boundaries"));
        assert!(aggregate.metadata.user_query.contains("GUI interface"));
        
        println!("✅ Complete CIM creation workflow validated through SAGE domain model");
        println!("✅ Session reached Completed state through proper event sourcing");
        println!("✅ All expert coordination events recorded for audit and replay");
    }

    #[test]
    fn test_failed_workflow_recovery() {
        // Given: A session that encounters an error during processing
        let session_id = SageSessionId::new();
        let mut aggregate = SageOrchestratorAggregate::new(session_id.clone());
        
        // Start session normally
        let start_events = aggregate.handle_command(SageCommand::StartSession {
            user_query: "Invalid or malformed query that cannot be parsed".to_string(),
            context: HashMap::new(),
            selected_expert: None,
        }).unwrap();
        
        for event in start_events {
            aggregate.apply_event(event).unwrap();
        }
        
        // Simulate failure during analysis
        let failure_event = SageEvent::SessionFailed {
            session_id: session_id.clone(),
            error: "Query analysis failed: Unable to identify domain or orchestration requirements".to_string(),
            timestamp: Utc::now(),
        };
        
        // When: Apply failure event
        let result = aggregate.apply_event(failure_event);
        
        // Then: Verify proper failure handling
        assert!(result.is_ok());
        assert!(matches!(aggregate.state, SageSessionState::Failed { .. }));
        
        if let SageSessionState::Failed { error } = &aggregate.state {
            assert!(error.contains("Query analysis failed"));
        }
        
        // Verify session failure is properly recorded in event history
        let events = aggregate.events();
        assert!(events.iter().any(|e| matches!(e, SageEvent::SessionFailed { .. })));
        
        println!("✅ Session failure handling validated");
        println!("✅ Error state properly maintained with descriptive error message");
    }
}