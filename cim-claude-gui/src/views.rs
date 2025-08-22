/*
 * Copyright 2025 - Cowboy AI, LLC.
 * All rights reserved.
 */

//! View components for the CIM Manager GUI
//! 
//! This module contains reusable UI components for different parts of the interface.
//! Components follow Iced's widget pattern for composability.

use iced::{Element, Length, Color, widget::{text, container, button, text_input, scrollable, column, row, Space}};
use crate::messages::{Message, ConversationMessage, MessageRole};

/// Renders a single conversation message with proper styling
pub fn message_view(message: &ConversationMessage) -> Element<Message> {
    let role_info = match message.role {
        MessageRole::User => ("🙋", "You", Color::from_rgb(0.2, 0.6, 1.0)),
        MessageRole::Assistant => ("🤖", "Claude", Color::from_rgb(0.4, 0.8, 0.4)),
        MessageRole::Sage => ("🎭", "SAGE", Color::from_rgb(0.8, 0.4, 0.8)),
        MessageRole::System => ("⚙️", "System", Color::from_rgb(0.6, 0.6, 0.6)),
    };
    
    let (role_icon, role_text, role_color) = role_info;
    
    container(
        column![
            // Message header
            row![
                text(format!("{} {}", role_icon, role_text))
                    .size(12)
                    .color(role_color),
                Space::with_width(Length::Fill),
                text(message.timestamp.format("%H:%M:%S").to_string())
                    .size(10)
                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
            ].spacing(5),
            
            // Message content
            container(
                text(&message.content).size(14)
            )
            .padding([8, 12])
            .width(Length::Fill),
            
            // Agent info if available
            if let Some(ref agent) = message.agent_name {
                text(format!("Expert: {}", agent))
                    .size(10)
                    .color(Color::from_rgb(0.4, 0.4, 0.4))
            } else {
                text("")
                    .size(10)
            }
        ].spacing(3)
    )
    .padding([12, 16])
    .width(Length::Fill)
    .style(container::bordered_box)
    .into()
}

/// Renders a list of conversation messages with scrolling
pub fn conversation_history_view(messages: &[ConversationMessage]) -> Element<Message> {
    if messages.is_empty() {
        container(
            column![
                text("💬 No messages yet")
                    .size(16)
                    .color(Color::from_rgb(0.6, 0.6, 0.6)),
                text("Start the conversation by typing a message below.")
                    .size(12)
                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
            ].spacing(5)
        )
        .padding(20)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .height(Length::Fixed(200.0))
        .into()
    } else {
        scrollable(
            column(
                messages.iter()
                    .map(|msg| message_view(msg))
                    .collect::<Vec<_>>()
            ).spacing(8)
        )
        .height(Length::Fixed(400.0))
        .into()
    }
}

/// Message input component with send button
pub fn message_input_view(
    conversation_id: String,
    input_value: &str
) -> Element<Message> {
    row![
        text_input("Type your message here...", input_value)
            .on_input(Message::MessageInputChanged)
            .on_submit(Message::SendMessage {
                conversation_id: conversation_id.clone(),
                message: input_value.to_string(),
            })
            .padding(12)
            .size(14)
            .width(Length::Fill),
        
        button(
            row![
                text("➤").size(14),
                text("Send").size(14),
            ].spacing(5)
        )
        .on_press(Message::SendMessage {
            conversation_id,
            message: input_value.to_string(),
        })
        .padding([12, 20])
        .style(button::primary),
    ].spacing(10)
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

/// Connection status indicator
pub fn connection_status_view(connected: bool, error: Option<&String>) -> Element<Message> {
    let (status_icon, status_text, status_color) = if connected {
        ("🟢", "Connected", Color::from_rgb(0.2, 0.8, 0.2))
    } else {
        ("🔴", "Disconnected", Color::from_rgb(0.8, 0.2, 0.2))
    };
    
    let mut content = vec![
        text(format!("{} {}", status_icon, status_text))
            .size(12)
            .color(status_color)
            .into()
    ];
    
    if let Some(error_msg) = error {
        content.push(
            text(format!("Error: {}", error_msg))
                .size(10)
                .color(Color::from_rgb(0.8, 0.4, 0.4))
                .into()
        );
    }
    
    column(content)
        .spacing(3)
        .into()
}

/// Conversation list item
pub fn conversation_item_view(
    conversation_id: String, 
    message_count: usize,
    is_selected: bool
) -> Element<'static, Message> {
    let button_style = if is_selected {
        button::primary
    } else {
        button::secondary
    };
    
    button(
        row![
            column![
                text(conversation_id.clone())
                    .size(14),
                text(format!("{} messages", message_count))
                    .size(10)
                    .color(Color::from_rgb(0.6, 0.6, 0.6)),
            ].spacing(2),
            Space::with_width(Length::Fill),
            if is_selected {
                text("▶")
                    .size(12)
                    .color(Color::from_rgb(0.2, 0.6, 1.0))
            } else {
                text("▶")
                    .size(12)
                    .color(Color::from_rgb(0.5, 0.5, 0.5))
            },
        ].spacing(10)
        .align_y(iced::alignment::Vertical::Center)
    )
    .on_press(Message::ConversationSelected(conversation_id))
    .width(Length::Fill)
    .padding([12, 16])
    .style(button_style)
    .into()
}

pub fn placeholder_view() -> Element<'static, Message> {
    text("View components now available!").into()
}