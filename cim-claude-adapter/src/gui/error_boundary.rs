/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! Error boundary and user feedback system for reactive UI components

use iced::{
    widget::{button, column, container, row, text, Space},
    Element, Length, Color, alignment,
    theme::{self, Text},
};
use std::fmt;

use crate::gui::messages::{Message, LoadingState};

/// Error severity levels for appropriate user feedback
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl ErrorSeverity {
    pub fn color(&self) -> Color {
        match self {
            Self::Info => Color::from_rgb(0.2, 0.6, 1.0),      // Blue
            Self::Warning => Color::from_rgb(1.0, 0.6, 0.0),   // Orange
            Self::Error => Color::from_rgb(1.0, 0.2, 0.2),     // Red
            Self::Critical => Color::from_rgb(0.8, 0.0, 0.2),  // Dark Red
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Info => "ℹ️",
            Self::Warning => "⚠️",
            Self::Error => "❌",
            Self::Critical => "🚨",
        }
    }
    
    pub fn title(&self) -> &'static str {
        match self {
            Self::Info => "Information",
            Self::Warning => "Warning",
            Self::Error => "Error",
            Self::Critical => "Critical Error",
        }
    }
}

/// Structured error information for user display
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub severity: ErrorSeverity,
    pub title: String,
    pub message: String,
    pub details: Option<String>,
    pub recovery_action: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ErrorInfo {
    pub fn new(severity: ErrorSeverity, title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity,
            title: title.into(),
            message: message.into(),
            details: None,
            recovery_action: None,
            timestamp: chrono::Utc::now(),
        }
    }
    
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
    
    pub fn with_recovery_action(mut self, action: impl Into<String>) -> Self {
        self.recovery_action = Some(action.into());
        self
    }
    
    /// Create error from bridge connection issues
    pub fn bridge_connection_error(message: impl Into<String>) -> Self {
        Self::new(
            ErrorSeverity::Error,
            "Bridge Connection Failed",
            message,
        )
        .with_details("The TEA-ECS bridge could not establish connection to the message bus")
        .with_recovery_action("Check NATS server status and retry connection")
    }
    
    /// Create error for Claude API issues
    pub fn claude_api_error(message: impl Into<String>) -> Self {
        Self::new(
            ErrorSeverity::Warning,
            "Claude API Issue",
            message,
        )
        .with_details("Communication with Claude API encountered an error")
        .with_recovery_action("Verify API key and network connectivity")
    }
    
    /// Create error for conversation management issues
    pub fn conversation_error(message: impl Into<String>) -> Self {
        Self::new(
            ErrorSeverity::Error,
            "Conversation Error",
            message,
        )
        .with_details("An error occurred while managing conversation state")
        .with_recovery_action("Try refreshing the conversation or starting a new one")
    }
    
    /// Create error for message sending issues
    pub fn message_send_error(message: impl Into<String>) -> Self {
        Self::new(
            ErrorSeverity::Warning,
            "Message Send Failed",
            message,
        )
        .with_details("Your message could not be sent to the conversation")
        .with_recovery_action("Check connection status and try sending again")
    }
    
    /// Create info message for successful operations
    pub fn success_info(message: impl Into<String>) -> Self {
        Self::new(
            ErrorSeverity::Info,
            "Success",
            message,
        )
    }
}

impl fmt::Display for ErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.severity.icon(), self.title, self.message)
    }
}

/// Error boundary component for wrapping UI sections
pub struct ErrorBoundary {
    error: Option<ErrorInfo>,
    show_details: bool,
}

impl ErrorBoundary {
    pub fn new() -> Self {
        Self {
            error: None,
            show_details: false,
        }
    }
    
    pub fn with_error(error: ErrorInfo) -> Self {
        Self {
            error: Some(error),
            show_details: false,
        }
    }
    
    /// Render error boundary with optional fallback content
    pub fn view<'a>(
        &self,
        content: Element<'a, Message>,
        fallback: Option<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        if let Some(ref error) = self.error {
            // Show error UI
            column![
                self.render_error_banner(error),
                if let Some(fallback_content) = fallback {
                    fallback_content
                } else {
                    self.render_error_content(error)
                }
            ]
            .spacing(10)
            .into()
        } else {
            // Show normal content
            content
        }
    }
    
    fn render_error_banner(&self, error: &ErrorInfo) -> Element<Message> {
        let banner_color = error.severity.color();
        
        container(
            row![
                text(error.severity.icon()).size(20),
                column![
                    text(&error.title)
                        .size(14)
                        .style(Text::Color(banner_color)),
                    text(&error.message)
                        .size(12),
                ]
                .spacing(2),
                Space::with_width(Length::Fill),
                row![
                    if self.show_details {
                        button("Hide Details")
                            .on_press(Message::ErrorDismissed)
                    } else {
                        button("Show Details")
                            .on_press(Message::ErrorOccurred("toggle_details".to_string()))
                    },
                    button("Dismiss")
                        .on_press(Message::ErrorDismissed),
                ]
                .spacing(5),
            ]
            .spacing(10)
            .align_y(alignment::Vertical::Center)
        )
        .padding(10)
        .style(theme::Container::Custom(Box::new(ErrorBannerStyle { color: banner_color })))
        .width(Length::Fill)
        .into()
    }
    
    fn render_error_content(&self, error: &ErrorInfo) -> Element<Message> {
        if self.show_details {
            column![
                if let Some(ref details) = error.details {
                    column![
                        text("Details:").size(12),
                        text(details).size(11),
                    ]
                    .spacing(5)
                    .into()
                } else {
                    Space::with_height(0).into()
                },
                
                if let Some(ref recovery) = error.recovery_action {
                    column![
                        text("Recovery Action:").size(12),
                        text(recovery).size(11),
                    ]
                    .spacing(5)
                    .into()
                } else {
                    Space::with_height(0).into()
                },
                
                text(format!("Occurred: {}", error.timestamp.format("%H:%M:%S UTC")))
                    .size(10)
                    .style(Text::Color(Color::from_rgb(0.5, 0.5, 0.5))),
            ]
            .spacing(10)
            .padding(10)
            .into()
        } else {
            Space::with_height(0).into()
        }
    }
}

impl Default for ErrorBoundary {
    fn default() -> Self {
        Self::new()
    }
}

/// Custom style for error banner background
struct ErrorBannerStyle {
    color: Color,
}

impl iced::widget::container::StyleSheet for ErrorBannerStyle {
    type Style = iced::Theme;
    
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color {
                r: self.color.r,
                g: self.color.g,
                b: self.color.b,
                a: 0.1,
            })),
            border: iced::Border {
                color: self.color,
                width: 1.0,
                radius: 5.0.into(),
            },
            text_color: None,
            shadow: iced::Shadow::default(),
        }
    }
}

/// Loading state indicator component
pub struct LoadingIndicator;

impl LoadingIndicator {
    /// Render loading state with appropriate UI feedback
    pub fn view(state: &LoadingState, content: Element<Message>) -> Element<Message> {
        match state {
            LoadingState::Idle => content,
            
            LoadingState::Loading => {
                column![
                    row![
                        text("⏳").size(16),
                        text("Loading...").size(14),
                        Space::with_width(Length::Fill),
                    ]
                    .spacing(5)
                    .align_y(alignment::Vertical::Center),
                    content,
                ]
                .spacing(5)
                .into()
            }
            
            LoadingState::Success => {
                // Could show a brief success indicator
                content
            }
            
            LoadingState::Error(error) => {
                let error_info = ErrorInfo::new(
                    ErrorSeverity::Error,
                    "Operation Failed",
                    error.clone(),
                );
                
                ErrorBoundary::with_error(error_info).view(
                    content,
                    Some(
                        column![
                            text("⚠️ An error occurred while loading this content")
                                .size(14)
                                .style(Text::Color(Color::from_rgb(0.7, 0.3, 0.3))),
                            button("Retry")
                                .on_press(Message::ErrorDismissed), // Would trigger retry
                        ]
                        .spacing(10)
                        .align_x(alignment::Horizontal::Center)
                        .into()
                    )
                )
            }
        }
    }
}

/// Toast notification system for non-blocking feedback
pub struct ToastNotification {
    pub message: String,
    pub severity: ErrorSeverity,
    pub duration_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ToastNotification {
    pub fn new(severity: ErrorSeverity, message: impl Into<String>) -> Self {
        Self {
            severity,
            message: message.into(),
            duration_ms: match severity {
                ErrorSeverity::Info => 3000,
                ErrorSeverity::Warning => 5000,
                ErrorSeverity::Error => 7000,
                ErrorSeverity::Critical => 0, // Don't auto-dismiss critical errors
            },
            timestamp: chrono::Utc::now(),
        }
    }
    
    pub fn is_expired(&self) -> bool {
        if self.duration_ms == 0 {
            return false; // Never expires
        }
        
        let elapsed = chrono::Utc::now() - self.timestamp;
        elapsed.num_milliseconds() > self.duration_ms as i64
    }
    
    pub fn view(&self) -> Element<Message> {
        let color = self.severity.color();
        
        container(
            row![
                text(self.severity.icon()).size(16),
                text(&self.message).size(12),
                Space::with_width(Length::Fill),
                button("✕")
                    .on_press(Message::ErrorDismissed)
                    .style(theme::Button::Secondary),
            ]
            .spacing(10)
            .align_y(alignment::Vertical::Center)
        )
        .padding(10)
        .style(theme::Container::Custom(Box::new(ToastStyle { color })))
        .width(Length::Fill)
        .into()
    }
}

/// Custom style for toast notifications
struct ToastStyle {
    color: Color,
}

impl iced::widget::container::StyleSheet for ToastStyle {
    type Style = iced::Theme;
    
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color {
                r: self.color.r,
                g: self.color.g,
                b: self.color.b,
                a: 0.9,
            })),
            border: iced::Border {
                color: Color::WHITE,
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: Some(Color::WHITE),
            shadow: iced::Shadow {
                color: Color::BLACK,
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 10.0,
            },
        }
    }
}

/// Utility functions for creating common error scenarios
pub mod errors {
    use super::*;
    
    pub fn bridge_disconnected() -> ErrorInfo {
        ErrorInfo::bridge_connection_error("Bridge connection lost")
            .with_details("The connection to the TEA-ECS bridge has been interrupted")
            .with_recovery_action("The system will attempt to reconnect automatically")
    }
    
    pub fn nats_unavailable() -> ErrorInfo {
        ErrorInfo::new(
            ErrorSeverity::Critical,
            "NATS Server Unavailable",
            "Cannot connect to the message bus",
        )
        .with_details("The NATS message server is not responding")
        .with_recovery_action("Check if NATS server is running and accessible")
    }
    
    pub fn claude_api_limit() -> ErrorInfo {
        ErrorInfo::claude_api_error("API rate limit exceeded")
            .with_details("Too many requests sent to Claude API")
            .with_recovery_action("Wait a moment before sending more messages")
    }
    
    pub fn invalid_conversation() -> ErrorInfo {
        ErrorInfo::conversation_error("Invalid conversation state")
            .with_details("The selected conversation is in an invalid state")
            .with_recovery_action("Try selecting a different conversation or restart the application")
    }
    
    pub fn message_too_long() -> ErrorInfo {
        ErrorInfo::message_send_error("Message exceeds maximum length")
            .with_details("Claude API has limits on message length")
            .with_recovery_action("Try breaking your message into smaller parts")
    }
}