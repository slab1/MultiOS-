//! Authentication System Usage Examples
//! 
//! This module demonstrates how to use the MultiOS authentication system
//! with various authentication methods and configurations.

#![allow(dead_code)]

use crate::security::{
    AuthManager, AuthConfig, AuthMethod, AuthError, AuthResult,
    SessionToken, PasswordPolicy, BiometricData, TOTPConfig, 
    SMSConfig, init_auth_manager, get_auth_manager, AuthMiddleware,
    AuthMiddlewareConfig
};

/// Example configuration for standard user authentication
pub fn create_standard_auth_config() -> AuthConfig {
    AuthConfig {
        session_timeout_minutes: 30,
        max_concurrent_sessions: 3,
        max_failed_attempts: 5,
        lockout_duration_minutes: 15,
        rate_limit_requests_per_hour: 100,
        require_multi_factor: false,
        allowed_auth_methods: vec![
            AuthMethod::Password,
            AuthMethod::TOTP,
        ],
        biometric_timeout_seconds: 30,
        password_policy: PasswordPolicy {
            min_length: 8,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_digits: true,
            require_symbols: false,
            require_non_alphabetic: true,
            prevent_common_passwords: true,
            prevent_user_info: true,
            max_age_days: Some(90),
            min_age_days: 1,
            history_count: 5,
            complexity_score_required: 3,
        },
        audit_successful_logins: true,
        audit_failed_logins: true,
        session_persistence: true,
    }
}

/// Example configuration for admin authentication (more strict)
pub fn create_admin_auth_config() -> AuthConfig {
    AuthConfig {
        session_timeout_minutes: 15,
        max_concurrent_sessions: 1,
        max_failed_attempts: 3,
        lockout_duration_minutes: 60,
        rate_limit_requests_per_hour: 50,
        require_multi_factor: true,
        allowed_auth_methods: vec![
            AuthMethod::Password,
            AuthMethod::TOTP,
            AuthMethod::BiometricFingerprint,
            AuthMethod::HardwareToken,
        ],
        biometric_timeout_seconds: 15,
        password_policy: PasswordPolicy {
            min_length: 12,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_digits: true,
            require_symbols: true,
            require_non_alphabetic: true,
            prevent_common_passwords: true,
            prevent_user_info: true,
            max_age_days: Some(30),
            min_age_days: 7,
            history_count: 10,
            complexity_score_required: 4,
        },
        audit_successful_logins: true,
        audit_failed_logins: true,
        session_persistence: true,
    }
}

/// Example: Initialize authentication system
pub fn example_init_authentication() -> AuthResult<()> {
    let config = create_standard_auth_config();
    init_auth_manager(config)?;
    println!("Authentication system initialized successfully");
    Ok(())
}

/// Example: Basic password authentication
pub fn example_password_auth() -> AuthResult<SessionToken> {
    let auth_manager = get_auth_manager()
        .and_then(|mgr| mgr.lock().as_ref().ok().and_then(|a| Some(a.0.as_ref()?)))
        .ok_or(AuthError::NotInitialized)?;

    // Authenticate user with password
    let session = auth_manager.authenticate_password("john_doe", "SecurePass123!", Some("192.168.1.100"))?;
    println!("User authenticated successfully: {}", session.token_id);
    Ok(session)
}

/// Example: Biometric authentication setup and usage
pub fn example_biometric_auth() -> AuthResult<SessionToken> {
    let auth_manager = get_auth_manager()
        .and_then(|mgr| mgr.lock().as_ref().ok().and_then(|a| Some(a.0.as_ref()?)))
        .ok_or(AuthError::NotInitialized)?;

    let user_id = 1001; // Example user ID
    
    // Enroll biometric template (this would come from actual biometric hardware)
    let fingerprint_template = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
    auth_manager.enroll_biometric(user_id, AuthMethod::BiometricFingerprint, &fingerprint_template, 95)?;
    
    // Authenticate with biometric
    let session = auth_manager.authenticate_biometric(
        user_id, 
        AuthMethod::BiometricFingerprint, 
        &fingerprint_template, 
        Some("192.168.1.100")
    )?;
    println!("Biometric authentication successful: {}", session.token_id);
    Ok(session)
}

/// Example: TOTP setup and authentication
pub fn example_totp_auth() -> AuthResult<SessionToken> {
    let auth_manager = get_auth_manager()
        .and_then(|mgr| mgr.lock().as_ref().ok().and_then(|a| Some(a.0.as_ref()?)))
        .ok_or(AuthError::NotInitialized)?;

    let user_id = 1001;
    
    // Setup TOTP (in real usage, this would be a secure random secret)
    let totp_secret = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x57, 0x6f, 0x72, 0x6c, 0x64];
    auth_manager.setup_totp(user_id, &totp_secret, "SHA1", 6, 30)?;
    
    // Authenticate with TOTP code
    let totp_code = "123456"; // This would come from user's authenticator app
    let session = auth_manager.authenticate_totp(user_id, totp_code, Some("192.168.1.100"))?;
    println!("TOTP authentication successful: {}", session.token_id);
    Ok(session)
}

/// Example: Multi-factor authentication
pub fn example_multi_factor_auth() -> AuthResult<SessionToken> {
    let auth_manager = get_auth_manager()
        .and_then(|mgr| mgr.lock().as_ref().ok().and_then(|a| Some(a.0.as_ref()?)))
        .ok_or(AuthError::NotInitialized)?;

    let username = "admin_user";
    
    // Start multi-factor authentication process
    let challenge_id = auth_manager.start_multi_factor_auth(username, AuthMethod::Password, Some("192.168.1.100"))?;
    println!("MFA challenge started: {}", challenge_id);
    
    // Complete multi-factor authentication with multiple factors
    let verification_data = vec![
        b"123456".to_vec(),           // TOTP code
        vec![0x12, 0x34, 0x56],       // Biometric template
        b"token123".to_vec(),         // Hardware token ID
    ];

    let session = auth_manager.complete_multi_factor_auth(
        &challenge_id,
        vec![
            AuthMethod::TOTP,
            AuthMethod::BiometricFingerprint,
            AuthMethod::HardwareToken,
        ],
        verification_data,
        Some("192.168.1.100")
    )?;
    println!("Multi-factor authentication successful: {}", session.token_id);
    Ok(session)
}

/// Example: Session management
pub fn example_session_management() -> AuthResult<()> {
    let auth_manager = get_auth_manager()
        .and_then(|mgr| mgr.lock().as_ref().ok().and_then(|a| Some(a.0.as_ref()?)))
        .ok_or(AuthError::NotInitialized)?;

    let user_id = 1001;
    
    // Create a new session
    let session = auth_manager.create_session(
        user_id,
        vec![AuthMethod::Password],
        Some("192.168.1.100"),
        Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
    )?;
    println!("Session created: {}", session.token_id);
    
    // Validate the session
    let validated_session = auth_manager.validate_session(&session.token_id)?;
    println!("Session validated, expires at: {}", validated_session.expires_at);
    
    // Get active session count
    let session_count = auth_manager.get_active_session_count();
    println!("Active sessions: {}", session_count);
    
    // Expire the session
    auth_manager.expire_session(&session.token_id)?;
    println!("Session expired: {}", session.token_id);
    
    Ok(())
}

/// Example: Password management
pub fn example_password_management() -> AuthResult<()> {
    let auth_manager = get_auth_manager()
        .and_then(|mgr| mgr.lock().as_ref().ok().and_then(|a| Some(a.0.as_ref()?)))
        .ok_or(AuthError::NotInitialized)?;

    let user_id = 1001;
    
    // Hash a password
    let (password_hash, salt) = auth_manager.hash_password("NewSecurePass123!", None)?;
    println!("Password hashed successfully");
    
    // Verify the password
    let is_valid = auth_manager.verify_password("NewSecurePass123!", &password_hash, &salt)?;
    println!("Password verification result: {}", is_valid);
    
    // Change password (this would integrate with user management system)
    // auth_manager.change_password(user_id, "old_password", "new_password")?;
    
    Ok(())
}

/// Example: Security features - account lockout
pub fn example_security_features() -> AuthResult<()> {
    let auth_manager = get_auth_manager()
        .and_then(|mgr| mgr.lock().as_ref().ok().and_then(|a| Some(a.0.as_ref()?)))
        .ok_or(AuthError::NotInitialized)?;

    let user_id = 1001;
    
    // Simulate failed authentication attempts
    for i in 0..6 {
        let result = auth_manager.authenticate_password("user", "wrong_password", Some("192.168.1.100"));
        match result {
            Ok(_) => println!("Unexpected success on attempt {}", i + 1),
            Err(AuthError::InvalidCredentials) => println!("Failed attempt {}", i + 1),
            Err(AuthError::AccountLocked) => {
                println!("Account locked after {} attempts", i + 1);
                break;
            }
            Err(e) => println!("Other error: {:?}", e),
        }
    }
    
    // Unlock account (this would typically require admin privileges)
    auth_manager.unlock_account(user_id)?;
    println!("Account unlocked for user {}", user_id);
    
    // Cleanup expired sessions
    let cleaned_count = auth_manager.cleanup_expired_sessions()?;
    println!("Cleaned up {} expired sessions", cleaned_count);
    
    Ok(())
}

/// Example: Authentication middleware
pub fn example_auth_middleware() -> AuthResult<()> {
    // Setup middleware configuration
    let middleware_config = AuthMiddlewareConfig {
        require_authentication: true,
        required_auth_methods: vec![AuthMethod::Password],
        session_validation_required: true,
        context_validation_required: true,
        audit_required: true,
        redirect_on_failure: false,
        failure_redirect_url: None,
    };

    let middleware = AuthMiddleware::new(middleware_config);
    
    // Check if authentication is required
    if middleware.requires_authentication() {
        println!("Authentication is required for this operation");
    }
    
    // Check if specific auth method is allowed
    if middleware.is_auth_method_allowed(AuthMethod::Password) {
        println!("Password authentication is allowed");
    }
    
    // Validate session through middleware
    // (This would require a valid session token in real usage)
    // let session = middleware.validate_session("some_session_token")?;
    
    Ok(())
}

/// Example: Hardware token authentication
pub fn example_hardware_token_auth() -> AuthResult<SessionToken> {
    let auth_manager = get_auth_manager()
        .and_then(|mgr| mgr.lock().as_ref().ok().and_then(|a| Some(a.0.as_ref()?)))
        .ok_or(AuthError::NotInitialized)?;

    // In a real implementation, hardware tokens would be pre-registered
    // This is a simplified example
    let token_id = "TOKEN_ABC123";
    let challenge = Some(b"challenge_data");
    
    let session = auth_manager.authenticate_hardware_token(token_id, challenge, Some("192.168.1.100"))?;
    println!("Hardware token authentication successful: {}", session.token_id);
    Ok(session)
}

/// Example: SMS authentication setup
pub fn example_sms_auth_setup() -> AuthResult<()> {
    let auth_manager = get_auth_manager()
        .and_then(|mgr| mgr.lock().as_ref().ok().and_then(|a| Some(a.0.as_ref()?)))
        .ok_or(AuthError::NotInitialized)?;

    let user_id = 1001;
    
    // Setup SMS authentication
    auth_manager.setup_sms(user_id, "+1234567890")?;
    println!("SMS authentication setup for user {}", user_id);
    
    // Send SMS verification code
    let code = auth_manager.send_sms_code(user_id)?;
    println!("SMS code sent: {} (in real implementation, this would be sent via SMS)", code);
    
    Ok(())
}

/// Example: Get authentication statistics
pub fn example_auth_statistics() -> AuthResult<()> {
    let auth_manager = get_auth_manager()
        .and_then(|mgr| mgr.lock().as_ref().ok().and_then(|a| Some(a.0.as_ref()?)))
        .ok_or(AuthError::NotInitialized)?;

    let stats = auth_manager.get_stats();
    
    println!("Authentication Statistics:");
    println!("  Total login attempts: {}", stats.total_login_attempts);
    println!("  Successful logins: {}", stats.successful_logins);
    println!("  Failed logins: {}", stats.failed_logins);
    println!("  Locked accounts: {}", stats.locked_accounts);
    println!("  Active sessions: {}", stats.active_sessions);
    println!("  Multi-factor successes: {}", stats.multi_factor_successes);
    println!("  Multi-factor failures: {}", stats.multi_factor_failures);
    println!("  Biometric attempts: {}", stats.biometric_attempts);
    println!("  Biometric successes: {}", stats.biometric_successes);
    println!("  Rate limit triggers: {}", stats.rate_limit_triggers);
    
    Ok(())
}

/// Complete authentication flow example
pub fn example_complete_auth_flow() -> AuthResult<()> {
    println!("=== Complete Authentication Flow Example ===");
    
    // Initialize authentication system
    example_init_authentication()?;
    
    // Demonstrate different authentication methods
    let _password_session = example_password_auth()?;
    let _biometric_session = example_biometric_auth()?;
    let _totp_session = example_totp_auth()?;
    let _mfa_session = example_multi_factor_auth()?;
    
    // Demonstrate session management
    example_session_management()?;
    
    // Demonstrate password management
    example_password_management()?;
    
    // Demonstrate security features
    example_security_features()?;
    
    // Demonstrate middleware
    example_auth_middleware()?;
    
    // Demonstrate hardware token
    let _hardware_session = example_hardware_token_auth()?;
    
    // Demonstrate SMS setup
    example_sms_auth_setup()?;
    
    // Show statistics
    example_auth_statistics()?;
    
    println!("=== Authentication Flow Complete ===");
    Ok(())
}

/// Benchmark authentication performance
pub fn example_auth_performance_benchmark() -> AuthResult<()> {
    let auth_manager = get_auth_manager()
        .and_then(|mgr| mgr.lock().as_ref().ok().and_then(|a| Some(a.0.as_ref()?)))
        .ok_or(AuthError::NotInitialized)?;

    println!("=== Authentication Performance Benchmark ===");
    
    // Benchmark password hashing
    let start_time = crate::hal::timers::get_system_time_ms();
    for _ in 0..1000 {
        let _ = auth_manager.hash_password("test_password_123", None);
    }
    let hash_time = crate::hal::timers::get_system_time_ms() - start_time;
    println!("Password hashing (1000 iterations): {}ms", hash_time);
    
    // Benchmark password verification
    let (hash, salt) = auth_manager.hash_password("test_password_123", None)?;
    let start_time = crate::hal::timers::get_system_time_ms();
    for _ in 0..1000 {
        let _ = auth_manager.verify_password("test_password_123", &hash, &salt);
    }
    let verify_time = crate::hal::timers::get_system_time_ms() - start_time;
    println!("Password verification (1000 iterations): {}ms", verify_time);
    
    // Benchmark session creation
    let start_time = crate::hal::timers::get_system_time_ms();
    for i in 0..100 {
        let _ = auth_manager.create_session(i as u32, vec![AuthMethod::Password], None, None);
    }
    let session_time = crate::hal::timers::get_system_time_ms() - start_time;
    println!("Session creation (100 iterations): {}ms", session_time);
    
    println!("=== Benchmark Complete ===");
    Ok(())
}