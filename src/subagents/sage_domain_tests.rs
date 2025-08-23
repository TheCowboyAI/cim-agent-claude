//! SAGE Domain Model Unit Tests
//!
//! Unit tests for SAGE domain model functionality that can run
//! without external dependencies like NATS or OpenSSL.

#[cfg(test)]
mod tests {
    use crate::subagents::sage::{
        SageOrchestratorAggregate, SageSessionId, SageSessionState, SageSessionMetadata,
        SageEvent, SageCommand, SageSessionStateMachine
    };
    use std::collections::HashMap;
    use chrono::Utc;

    #[test]
    fn test_sage_session_id_creation() {
        // Given/When
        let id1 = SageSessionId::new();
        let id2 = SageSessionId::new();
        
        // Then
        assert_ne!(id1, id2);
        assert!(!id1.as_str().is_empty());
    }

    #[test]
    fn test_sage_state_machine_valid_transitions() {
        use crate::subagents::sage::SageSessionState::*;
        
        // Valid transitions
        assert!(SageSessionStateMachine::can_transition(&Created, &AnalyzingQuery));
        assert!(SageSessionStateMachine::can_transition(&AnalyzingQuery, &RoutingToExperts));
        assert!(SageSessionStateMachine::can_transition(&ProcessingWithExperts, &SynthesizingResponse));
        assert!(SageSessionStateMachine::can_transition(&SynthesizingResponse, &Completed));
    }

    #[test]
    fn test_sage_state_machine_invalid_transitions() {
        use crate::subagents::sage::SageSessionState::*;
        
        // Invalid transitions
        assert!(!SageSessionStateMachine::can_transition(&Created, &ProcessingWithExperts));
        assert!(!SageSessionStateMachine::can_transition(&Completed, &Created));
    }

    #[test]
    fn test_sage_aggregate_creation() {
        // Given
        let session_id = SageSessionId::new();
        
        // When
        let aggregate = SageOrchestratorAggregate::new(session_id.clone());
        
        // Then
        assert_eq!(aggregate.aggregate_id(), &session_id);
        assert_eq!(aggregate.state, SageSessionState::Created);
        assert_eq!(aggregate.version(), 0);
    }

    #[test]
    fn test_sage_event_application() {
        // Given
        let session_id = SageSessionId::new();
        let mut aggregate = SageOrchestratorAggregate::new(session_id.clone());
        
        let metadata = SageSessionMetadata {
            user_query: "Test query".to_string(),
            selected_expert: None,
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
        assert_eq!(aggregate.metadata.user_query, "Test query");
    }

    #[test]
    fn test_sage_command_handling() {
        // Given
        let session_id = SageSessionId::new();
        let mut aggregate = SageOrchestratorAggregate::new(session_id.clone());
        
        let command = SageCommand::StartSession {
            user_query: "Create a CIM for inventory".to_string(),
            context: HashMap::new(),
            selected_expert: Some("ddd-expert".to_string()),
        };
        
        // When
        let result = aggregate.handle_command(command);
        
        // Then
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 2); // SessionCreated + QueryReceived
    }

    #[test]
    fn test_complete_sage_workflow() {
        // Given
        let session_id = SageSessionId::new();
        let mut aggregate = SageOrchestratorAggregate::new(session_id.clone());
        
        // When - Apply a sequence of events representing a complete workflow
        let events = vec![
            SageEvent::SessionCreated {
                session_id: session_id.clone(),
                metadata: SageSessionMetadata {
                    user_query: "Build healthcare CIM".to_string(),
                    selected_expert: None,
                    session_context: HashMap::new(),
                    created_at: Utc::now(),
                    last_updated: Utc::now(),
                },
                timestamp: Utc::now(),
            },
            SageEvent::QueryReceived {
                session_id: session_id.clone(),
                query: "Build healthcare CIM".to_string(),
                timestamp: Utc::now(),
            },
            SageEvent::QueryAnalyzed {
                session_id: session_id.clone(),
                routing_strategy: "multi_expert".to_string(),
                selected_experts: vec!["cim-expert".to_string(), "ddd-expert".to_string()],
                confidence_score: 0.9,
                timestamp: Utc::now(),
            },
            SageEvent::RoutedToExperts {
                session_id: session_id.clone(),
                experts: vec!["cim-expert".to_string(), "ddd-expert".to_string()],
                timestamp: Utc::now(),
            },
            // Add missing state transitions
            SageEvent::ExpertResponseReceived {
                session_id: session_id.clone(),
                expert_id: "cim-expert".to_string(),
                response: "CIM architecture guidance provided".to_string(),
                confidence_score: 0.9,
                timestamp: Utc::now(),
            },
            SageEvent::ResponseSynthesized {
                session_id: session_id.clone(),
                synthesized_response: "Complete healthcare CIM architecture with DDD boundaries".to_string(),
                contributing_experts: vec!["cim-expert".to_string(), "ddd-expert".to_string()],
                timestamp: Utc::now(),
            },
            SageEvent::SessionCompleted {
                session_id: session_id.clone(),
                duration_ms: 5000,
                timestamp: Utc::now(),
            },
        ];
        
        // Apply all events
        for event in events {
            let result = aggregate.apply_event(event);
            assert!(result.is_ok());
        }
        
        // Then
        assert_eq!(aggregate.state, SageSessionState::Completed);
        assert_eq!(aggregate.version(), 7); // Updated for 7 events
        assert_eq!(aggregate.events().len(), 7); // Updated for 7 events
        assert_eq!(aggregate.metadata.user_query, "Build healthcare CIM");
    }

    #[test]
    fn test_sage_failure_handling() {
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
        
        let failure_event = SageEvent::SessionFailed {
            session_id: session_id.clone(),
            error: "Query parsing failed".to_string(),
            timestamp: Utc::now(),
        };
        
        // When
        let result = aggregate.apply_event(failure_event);
        
        // Then
        assert!(result.is_ok());
        assert!(matches!(aggregate.state, SageSessionState::Failed { .. }));
    }

    #[test]
    fn test_session_id_validation() {
        // Given
        let session_id1 = SageSessionId::new();
        let session_id2 = SageSessionId::new();
        let mut aggregate = SageOrchestratorAggregate::new(session_id1.clone());
        
        // Event with wrong session ID
        let wrong_event = SageEvent::QueryReceived {
            session_id: session_id2, // Wrong ID
            query: "Test".to_string(),
            timestamp: Utc::now(),
        };
        
        // When
        let result = aggregate.apply_event(wrong_event);
        
        // Then
        assert!(result.is_err()); // Should reject event with wrong session ID
    }

    #[test] 
    fn test_domain_event_interface() {
        use cim_domain::DomainEvent;
        
        let session_id = SageSessionId::new();
        let event = SageEvent::SessionCreated {
            session_id: session_id.clone(),
            metadata: SageSessionMetadata {
                user_query: "test".to_string(),
                selected_expert: None,
                session_context: HashMap::new(),
                created_at: Utc::now(),
                last_updated: Utc::now(),
            },
            timestamp: Utc::now(),
        };
        
        // Test DomainEvent interface implementation
        assert_eq!(event.event_type(), "sage.session.created");
        assert!(event.subject().contains("cim.events.sage"));
        assert!(event.subject().contains(session_id.as_str()));
    }
}