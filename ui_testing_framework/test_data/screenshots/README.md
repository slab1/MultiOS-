# Current Screenshots Directory

This directory contains screenshots captured during test runs for comparison against baseline images.

## Structure

```
screenshots/
├── current/          # Latest test run screenshots
├── archive/          # Historical screenshots for trend analysis
└── failed/           # Screenshots from failed tests
```

## File Naming Convention

- Test name with timestamp: `login_test_20241201_143022.png`
- Sequential numbering: `dashboard_test_001.png`, `dashboard_test_002.png`
- Failure indicators: `login_test_FAILED_20241201_143022.png`

## Automatic Cleanup

Screenshots are automatically cleaned up based on retention policy:
- Current run: Always kept
- Archive: 30 days retention
- Failed tests: 7 days retention

## Integration with CI/CD

Screenshots are automatically uploaded to CI/CD artifacts for:
- Debugging failed tests
- Visual review of changes
- Historical trend analysis