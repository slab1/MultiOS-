# Visual Differences Directory

This directory contains visual diffs generated when screenshots don't match baseline images.

## Diff Types

### 1. Highlight Differences
- Uses color overlay to highlight changed pixels
- Red: Areas that differ significantly
- Green: Areas that are similar but not exact matches

### 2. Side-by-Side Comparison
- Baseline on left, current on right
- Easy visual identification of changes

### 3. Pixel-by-Pixel Diff
- Binary diff showing exact pixel differences
- Useful for pixel-perfect verification

### 4. Heatmap Diff
- Color intensity represents difference magnitude
- Warmer colors = larger differences

## File Naming

- Pattern: `{test_name}_diff_{timestamp}_{algorithm}.png`
- Example: `login_page_diff_20241201_143022_ssim.png`

## Algorithms Used

- **Pixel Perfect**: Exact pixel comparison
- **SSIM**: Structural Similarity Index
- **Perceptual Hash**: Perceptual hashing comparison
- **Histogram**: Color histogram comparison

## Analysis Tools

Use these tools to analyze diffs:
- `diff_viewer` - Interactive diff viewer
- `pixel_analyzer` - Pixel-level analysis
- `metrics_calculator` - Quantitative diff metrics

## Threshold Configuration

Diffs are generated based on configured thresholds:
- High sensitivity (0.02): Small changes highlighted
- Medium sensitivity (0.05): Moderate changes highlighted
- Low sensitivity (0.10): Only significant changes highlighted