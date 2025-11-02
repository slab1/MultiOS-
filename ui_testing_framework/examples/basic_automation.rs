//! Basic automation example showing how to interact with UI widgets
//! 
//! This example demonstrates:
//! - Starting the automation engine
//! - Finding and interacting with widgets
//! - Simulating user events
//! - Running automated test scenarios

use multios_ui_testing::*;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Basic UI Automation Example");
    
    // Initialize the automation engine
    let mut automation = AutomationEngine::new().await?;
    
    // Create a test scenario
    let scenario = TestScenario::new("basic_widget_interaction")
        .with_step(WidgetInteraction::find_by_id("login_button")?)
        .with_step(WidgetInteraction::click()?)
        .with_step(WidgetInteraction::find_by_id("username_field")?)
        .with_step(WidgetInteraction::type_text("test_user")?)
        .with_step(WidgetInteraction::find_by_id("password_field")?)
        .with_step(WidgetInteraction::type_text("test_password")?)
        .with_step(WidgetInteraction::find_by_id("submit_button")?)
        .with_step(WidgetInteraction::click()?);
    
    // Execute the scenario
    let result = automation.execute_scenario(scenario).await?;
    
    println!("Automation completed successfully: {:?}", result);
    
    // Clean up
    automation.shutdown().await?;
    
    Ok(())
}