use crate::feedback::UserFeedback;

#[test]
fn test_message_methods() {
    // These methods should not panic when called
    UserFeedback::success("Test success message");
    UserFeedback::error("Test error message");
    UserFeedback::warning("Test warning message");
    UserFeedback::info("Test info message");
    UserFeedback::tip("Test tip message");
}

#[test]
fn test_formatting_methods() {
    // These methods should not panic when called
    UserFeedback::section("Test Section");
    UserFeedback::separator();
}

#[test]
fn test_help_methods() {
    // These methods should not panic when called
    UserFeedback::show_command_help("capture");
    UserFeedback::show_command_help("serve");
    UserFeedback::show_command_help("list");
    UserFeedback::show_command_help("delete");
    UserFeedback::show_command_help("unknown");
}

#[test]
fn test_guide_methods() {
    // These methods should not panic when called
    UserFeedback::show_troubleshooting_guide();
    UserFeedback::show_system_requirements();
    UserFeedback::show_performance_tips();
}
