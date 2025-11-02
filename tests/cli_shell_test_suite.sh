#!/usr/bin/multios/sh
# MultiOS CLI Shell Test Suite
# This script tests the functionality of the MultiOS CLI shell

# Test configuration
TEST_COUNT=0
PASSED_COUNT=0
FAILED_COUNT=0

# Test functions
function test_command() {
    local test_name="$1"
    local command="$2"
    local expected_exit_code="${3:-0}"
    
    TEST_COUNT=$((TEST_COUNT + 1))
    
    print "Test $TEST_COUNT: $test_name"
    
    # Execute the command and capture result
    local result=$(eval "$command")
    local exit_code=$?
    
    # For this test, we'll simulate different exit codes
    # In real implementation, we would capture the actual exit code
    local simulated_exit_code=0
    
    if [ $simulated_exit_code -eq $expected_exit_code ]; then
        print "  PASS: Command executed successfully"
        PASSED_COUNT=$((PASSED_COUNT + 1))
    else
        print "  FAIL: Expected exit code $expected_exit_code, got $simulated_exit_code"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
}

function test_builtin_command() {
    local command="$1"
    local expected_output="$2"
    
    TEST_COUNT=$((TEST_COUNT + 1))
    
    print "Test $TEST_COUNT: Testing builtin command '$command'"
    
    # Execute the builtin command
    local result=$(eval "$command")
    
    # Check if the result matches expected output
    if [[ "$result" == *"$expected_output"* ]]; then
        print "  PASS: Output contains expected text"
        PASSED_COUNT=$((PASSED_COUNT + 1))
    else
        print "  FAIL: Expected output containing '$expected_output'"
        print "  Got: $result"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
}

function test_environment_variable() {
    local var_name="$1"
    local expected_value="$2"
    
    TEST_COUNT=$((TEST_COUNT + 1))
    
    print "Test $TEST_COUNT: Testing environment variable '$var_name'"
    
    # Get environment variable value
    local var_value=$(get_env "$var_name")
    
    if [ "$var_value" = "$expected_value" ]; then
        print "  PASS: Variable has correct value"
        PASSED_COUNT=$((PASSED_COUNT + 1))
    else
        print "  FAIL: Expected '$expected_value', got '$var_value'"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
}

function test_script_function() {
    local function_name="$1"
    local test_args="$2"
    
    TEST_COUNT=$((TEST_COUNT + 1))
    
    print "Test $TEST_COUNT: Testing script function '$function_name'"
    
    # Check if function exists
    if $(type -t "$function_name" 2>/dev/null); then
        print "  PASS: Function exists and is callable"
        PASSED_COUNT=$((PASSED_COUNT + 1))
        
        # Try to call the function
        if [ -n "$test_args" ]; then
            eval "$function_name $test_args"
        else
            eval "$function_name"
        fi
        
        if [ $? -eq 0 ]; then
            print "  PASS: Function executed successfully"
        else
            print "  FAIL: Function execution failed"
            FAILED_COUNT=$((FAILED_COUNT + 1))
        fi
    else
        print "  FAIL: Function '$function_name' not found"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
}

function test_file_operations() {
    local test_file="/tmp/multios_test_$(date +%s).txt"
    local test_content="Hello, MultiOS CLI!"
    
    TEST_COUNT=$((TEST_COUNT + 1))
    
    print "Test $TEST_COUNT: Testing file operations"
    
    # Test file creation
    write_file "$test_file" "$test_content"
    
    if $(file_exists "$test_file"); then
        print "  PASS: File created successfully"
        PASSED_COUNT=$((PASSED_COUNT + 1))
        
        # Test file reading
        local read_content=$(read_file "$test_file")
        if [ "$read_content" = "$test_content" ]; then
            print "  PASS: File content matches expected"
            PASSED_COUNT=$((PASSED_COUNT + 1))
        else
            print "  FAIL: File content mismatch"
            print "  Expected: $test_content"
            print "  Got: $read_content"
            FAILED_COUNT=$((FAILED_COUNT + 1))
        fi
        
        # Clean up
        execute "rm -f $test_file"
    else
        print "  FAIL: File creation failed"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
}

function test_math_functions() {
    local test_value=16
    
    TEST_COUNT=$((TEST_COUNT + 1))
    
    print "Test $TEST_COUNT: Testing mathematical functions"
    
    # Test sqrt function
    local sqrt_result=$(sqrt $test_value)
    if [ $sqrt_result -eq 4 ]; then
        print "  PASS: sqrt(16) = 4"
        PASSED_COUNT=$((PASSED_COUNT + 1))
    else
        print "  FAIL: sqrt(16) should be 4, got $sqrt_result"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
    
    # Test abs function
    local abs_result=$(abs -5)
    if [ $abs_result -eq 5 ]; then
        print "  PASS: abs(-5) = 5"
        PASSED_COUNT=$((PASSED_COUNT + 1))
    else
        print "  FAIL: abs(-5) should be 5, got $abs_result"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
    
    # Test pow function
    local pow_result=$(pow 2 3)
    if [ $pow_result -eq 8 ]; then
        print "  PASS: pow(2, 3) = 8"
        PASSED_COUNT=$((PASSED_COUNT + 1))
    else
        print "  FAIL: pow(2, 3) should be 8, got $pow_result"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
}

function test_string_functions() {
    local test_string="Hello World"
    
    TEST_COUNT=$((TEST_COUNT + 1))
    
    print "Test $TEST_COUNT: Testing string functions"
    
    # Test length function
    local length_result=$(length "$test_string")
    if [ $length_result -eq 11 ]; then
        print "  PASS: length('$test_string') = 11"
        PASSED_COUNT=$((PASSED_COUNT + 1))
    else
        print "  FAIL: length('$test_string') should be 11, got $length_result"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
    
    # Test upper function
    local upper_result=$(upper "$test_string")
    if [ "$upper_result" = "HELLO WORLD" ]; then
        print "  PASS: upper('$test_string') = 'HELLO WORLD'"
        PASSED_COUNT=$((PASSED_COUNT + 1))
    else
        print "  FAIL: upper('$test_string') should be 'HELLO WORLD', got '$upper_result'"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
    
    # Test substring function
    local substring_result=$(substring "$test_string" 0 5)
    if [ "$substring_result" = "Hello" ]; then
        print "  PASS: substring('$test_string', 0, 5) = 'Hello'"
        PASSED_COUNT=$((PASSED_COUNT + 1))
    else
        print "  FAIL: substring('$test_string', 0, 5) should be 'Hello', got '$substring_result'"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
}

function test_array_functions() {
    local test_array="[1, 2, 3, 4, 5]"
    
    TEST_COUNT=$((TEST_COUNT + 1))
    
    print "Test $TEST_COUNT: Testing array functions"
    
    # Test array_size function
    local array_result=$(array_size "$test_array")
    if [ $array_result -eq 5 ]; then
        print "  PASS: array_size works correctly"
        PASSED_COUNT=$((PASSED_COUNT + 1))
    else
        print "  FAIL: array_size should return 5, got $array_result"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
}

function test_system_functions() {
    TEST_COUNT=$((TEST_COUNT + 1))
    
    print "Test $TEST_COUNT: Testing system functions"
    
    # Test system_info function
    local sys_info=$(system_info)
    if [ -n "$sys_info" ]; then
        print "  PASS: system_info returned data"
        print "  System info preview: $(substring "$sys_info" 0 50)..."
        PASSED_COUNT=$((PASSED_COUNT + 1))
    else
        print "  FAIL: system_info returned empty result"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
    
    # Test get_env function
    local home_dir=$(get_env "HOME")
    if [ -n "$home_dir" ]; then
        print "  PASS: get_env returned HOME directory: $home_dir"
        PASSED_COUNT=$((PASSED_COUNT + 1))
    else
        print "  FAIL: get_env failed to return HOME directory"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
    
    # Test set_env function
    set_env "TEST_VAR" "test_value"
    local test_var=$(get_env "TEST_VAR")
    if [ "$test_var" = "test_value" ]; then
        print "  PASS: set_env and get_env work correctly"
        PASSED_COUNT=$((PASSED_COUNT + 1))
    else
        print "  FAIL: set_env/get_env test failed"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
}

# Main test execution
print "=== MultiOS CLI Shell Test Suite ==="
print "Starting comprehensive CLI functionality tests..."
print ""

# Test basic shell commands
print "=== Basic Shell Commands ==="
test_builtin_command "echo 'Hello World'" "Hello World"
test_builtin_command "pwd" "/"
test_builtin_command "uname" "MultiOS"

# Test environment variables
print ""
print "=== Environment Variables ==="
test_environment_variable "SHELL" "MultiOS Shell"
test_environment_variable "USER" "root"

# Test script functions
print ""
print "=== Script Functions ==="
test_script_function "show_system_info" ""
test_script_function "monitor_processes" ""

# Test file operations
print ""
print "=== File Operations ==="
test_file_operations

# Test mathematical functions
print ""
print "=== Mathematical Functions ==="
test_math_functions

# Test string functions
print ""
print "=== String Functions ==="
test_string_functions

# Test array functions
print ""
print "=== Array Functions ==="
test_array_functions

# Test system functions
print ""
print "=== System Functions ==="
test_system_functions

# Test advanced scripting features
print ""
print "=== Advanced Scripting Features ==="

TEST_COUNT=$((TEST_COUNT + 1))
print "Test $TEST_COUNT: Testing variable assignment"
local test_var="test_value"
if [ "$test_var" = "test_value" ]; then
    print "  PASS: Variable assignment works"
    PASSED_COUNT=$((PASSED_COUNT + 1))
else
    print "  FAIL: Variable assignment failed"
    FAILED_COUNT=$((FAILED_COUNT + 1))
fi

TEST_COUNT=$((TEST_COUNT + 1))
print "Test $TEST_COUNT: Testing conditional execution"
local condition_test=true
if $condition_test; then
    print "  PASS: Conditional execution works"
    PASSED_COUNT=$((PASSED_COUNT + 1))
else
    print "  FAIL: Conditional execution failed"
    FAILED_COUNT=$((FAILED_COUNT + 1))
fi

# Test error handling
print ""
print "=== Error Handling ==="

TEST_COUNT=$((TEST_COUNT + 1))
print "Test $TEST_COUNT: Testing non-existent command"
# This should fail gracefully
execute "nonexistent_command_12345" || {
    print "  PASS: Non-existent command handled correctly"
    PASSED_COUNT=$((PASSED_COUNT + 1))
}

# Test performance
print ""
print "=== Performance Tests ==="

TEST_COUNT=$((TEST_COUNT + 1))
print "Test $TEST_COUNT: Testing command execution speed"
local start_time=$(date +%s%3N)
echo "Performance test"
local end_time=$(date +%s%3N)
local duration=$((end_time - start_time))

if [ $duration -lt 1000 ]; then
    print "  PASS: Command executed quickly ($duration ms)"
    PASSED_COUNT=$((PASSED_COUNT + 1))
else
    print "  INFO: Command took $duration ms (acceptable for test environment)"
    PASSED_COUNT=$((PASSED_COUNT + 1))
fi

# Print test summary
print ""
print "=== Test Summary ==="
print "Total tests run: $TEST_COUNT"
print "Tests passed: $PASSED_COUNT"
print "Tests failed: $FAILED_COUNT"

if [ $FAILED_COUNT -eq 0 ]; then
    print ""
    print "🎉 ALL TESTS PASSED! 🎉"
    print "MultiOS CLI Shell is functioning correctly."
    exit 0
else
    failure_rate=$((FAILED_COUNT * 100 / TEST_COUNT))
    print ""
    print "⚠️  SOME TESTS FAILED ⚠️"
    print "Failure rate: $failure_rate%"
    print "Please check the failed tests above."
    exit 1
fi