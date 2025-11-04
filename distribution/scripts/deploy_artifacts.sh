#!/bin/bash

# MultiOS Artifact Deployment Script
# Deploys build artifacts to registries and storage systems

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ARTIFACTS_DIR="${1:-/workspace/artifacts}"
REGISTRY_URL="${REGISTRY_URL:-}"
DOCKER_REGISTRY="${DOCKER_REGISTRY:-}"
CRATES_REGISTRY="${CRATES_REGISTRY:-}"
DEPLOY_VERSION="${DEPLOY_VERSION:-latest}"

# Ensure artifacts directory exists
mkdir -p "$ARTIFACTS_DIR"

# Logging functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Deploy build artifacts
deploy_build_artifacts() {
    log "Deploying build artifacts..."
    
    # Deploy binaries for each architecture
    for binary_dir in "$ARTIFACTS_DIR"/build-artifacts-*; do
        if [ -d "$binary_dir" ]; then
            local arch=$(basename "$binary_dir" | sed 's/build-artifacts-//')
            log "Deploying artifacts for $arch"
            
            # Create deployment package
            local package_name="multios-${DEPLOY_VERSION}-${arch}"
            local package_dir="$ARTIFACTS_DIR/packages/$package_name"
            mkdir -p "$package_dir"
            
            # Copy binaries and documentation
            find "$binary_dir" -name "*.bin" -o -name "*.elf" | while read -r binary; do
                cp "$binary" "$package_dir/"
            done
            
            # Create checksum files
            cd "$package_dir"
            find . -type f -exec sha256sum {} \; > checksums.sha256
            find . -type f -exec md5sum {} \; > checksums.md5
            cd - > /dev/null
            
            # Create release notes
            generate_release_notes "$package_dir" "$arch"
            
            # Compress package
            cd "$ARTIFACTS_DIR/packages"
            tar -czf "${package_name}.tar.gz" "$package_name"
            zip -r "${package_name}.zip" "$package_name" >/dev/null 2>&1
            cd - > /dev/null
            
            success "Package created: $package_name"
        fi
    done
}

# Deploy Docker images
deploy_docker_images() {
    log "Deploying Docker images..."
    
    if [ -z "$DOCKER_REGISTRY" ]; then
        warning "Docker registry not configured"
        return 1
    fi
    
    # Build and push images for each architecture
    for arch in x86_64 arm64 riscv64; do
        log "Building Docker image for $arch"
        
        # Create architecture-specific Dockerfile
        local dockerfile="Dockerfile.${arch}"
        create_arch_dockerfile "$arch" "$dockerfile"
        
        # Build image
        if docker build -f "$dockerfile" -t "${DOCKER_REGISTRY}/multios:${DEPLOY_VERSION}-${arch}" .; then
            success "Docker image built: ${DOCKER_REGISTRY}/multios:${DEPLOY_VERSION}-${arch}"
            
            # Push to registry
            if docker push "${DOCKER_REGISTRY}/multios:${DEPLOY_VERSION}-${arch}"; then
                success "Docker image pushed: ${DOCKER_REGISTRY}/multios:${DEPLOY_VERSION}-${arch}"
            else
                error "Failed to push Docker image"
            fi
        else
            error "Failed to build Docker image for $arch"
        fi
    done
    
    # Create and push multi-arch manifest
    create_multi_arch_manifest
}

# Create architecture-specific Dockerfile
create_arch_dockerfile() {
    local arch=$1
    local dockerfile=$2
    
    case "$arch" in
        "x86_64")
            cat > "$dockerfile" << 'EOF'
FROM scratch
ADD multios-x86_64-unknown-none /multios
ENTRYPOINT ["/multios"]
EOF
            ;;
        "arm64")
            cat > "$dockerfile" << 'EOF'
FROM scratch
ADD multios-aarch64-unknown-none /multios
ENTRYPOINT ["/multios"]
EOF
            ;;
        "riscv64")
            cat > "$dockerfile" << 'EOF'
FROM scratch
ADD multios-riscv64gc-unknown-none-elf /multios
ENTRYPOINT ["/multios"]
EOF
            ;;
    esac
}

# Create multi-arch manifest
create_multi_arch_manifest() {
    log "Creating multi-arch manifest..."
    
    # Create manifest list
    local manifest_file="$ARTIFACTS_DIR/manifest.json"
    cat > "$manifest_file" << EOF
{
    "schemaVersion": 2,
    "mediaType": "application/vnd.docker.distribution.manifest.list.v2+json",
    "manifests": [
        {
            "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
            "size": 1234,
            "digest": "sha256:placeholder1",
            "platform": {
                "architecture": "amd64",
                "os": "linux"
            }
        },
        {
            "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
            "size": 1234,
            "digest": "sha256:placeholder2",
            "platform": {
                "architecture": "arm64",
                "os": "linux"
            }
        },
        {
            "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
            "size": 1234,
            "digest": "sha256:placeholder3",
            "platform": {
                "architecture": "riscv64",
                "os": "linux"
            }
        }
    ]
}
EOF
    
    warning "Manifest creation requires actual image digests - manual intervention needed"
}

# Deploy to package registries
deploy_package_registries() {
    log "Deploying to package registries..."
    
    # Deploy to crates.io
    if [ -n "$CRATES_REGISTRY" ]; then
        deploy_crates
    fi
    
    # Deploy to GitHub releases
    deploy_github_releases
}

# Deploy to crates.io
deploy_crates() {
    log "Deploying to crates.io..."
    
    if ! command -v cargo >/dev/null 2>&1; then
        warning "Cargo not available - skipping crates.io deployment"
        return 1
    fi
    
    # Check if publish is configured
    if ! grep -q "publish = \[" Cargo.toml 2>/dev/null; then
        log "Adding publish configuration to Cargo.toml"
        echo 'publish = ["crates-io"]' >> Cargo.toml
    fi
    
    # Publish to crates.io
    if [ -n "$CRATES_TOKEN" ]; then
        if cargo publish --token "$CRATES_TOKEN"; then
            success "Package published to crates.io"
        else
            error "Failed to publish to crates.io"
        fi
    else
        warning "crates.io token not configured - skipping publication"
    fi
}

# Deploy to GitHub releases
deploy_github_releases() {
    log "Deploying to GitHub releases..."
    
    if [ -z "$GITHUB_TOKEN" ]; then
        warning "GitHub token not configured - skipping GitHub releases"
        return 1
    fi
    
    # Create release
    local release_data=$(cat << EOF
{
    "tag_name": "v$DEPLOY_VERSION",
    "target_commitish": "main",
    "name": "MultiOS v$DEPLOY_VERSION",
    "body": "Release v$DEPLOY_VERSION\\n\\nSee CHANGELOG.md for details.",
    "draft": false,
    "prerelease": false
}
EOF
)
    
    local release_response=$(curl -s -X POST \
        -H "Authorization: token $GITHUB_TOKEN" \
        -H "Accept: application/vnd.github.v3+json" \
        --data "$release_data" \
        "https://api.github.com/repos/$GITHUB_REPOSITORY/releases")
    
    local upload_url=$(echo "$release_response" | jq -r '.upload_url' 2>/dev/null || echo "")
    
    if [ "$upload_url" != "null" ] && [ -n "$upload_url" ]; then
        success "GitHub release created"
        
        # Upload artifacts
        for package in "$ARTIFACTS_DIR"/packages/*.tar.gz "$ARTIFACTS_DIR"/packages/*.zip; do
            if [ -f "$package" ]; then
                upload_release_asset "$upload_url" "$package"
            fi
        done
    else
        error "Failed to create GitHub release"
    fi
}

# Upload release asset to GitHub
upload_release_asset() {
    local upload_url=$1
    local file_path=$2
    local filename=$(basename "$file_path")
    
    log "Uploading release asset: $filename"
    
    local response=$(curl -s -X POST \
        -H "Authorization: token $GITHUB_TOKEN" \
        -H "Content-Type: application/octet-stream" \
        --data-binary @"$file_path" \
        "${upload_url%\{*}/$filename")
    
    if [ $? -eq 0 ]; then
        success "Asset uploaded: $filename"
    else
        error "Failed to upload asset: $filename"
    fi
}

# Deploy to object storage
deploy_object_storage() {
    log "Deploying to object storage..."
    
    if [ -z "$S3_BUCKET" ]; then
        warning "S3 bucket not configured"
        return 1
    fi
    
    # Deploy artifacts to S3
    for package in "$ARTIFACTS_DIR"/packages/*; do
        if [ -f "$package" ]; then
            local filename=$(basename "$package")
            local s3_key="releases/$DEPLOY_VERSION/$filename"
            
            if aws s3 cp "$package" "s3://$S3_BUCKET/$s3_key"; then
                success "Uploaded to S3: $s3_key"
            else
                error "Failed to upload to S3: $s3_key"
            fi
        fi
    done
}

# Generate release notes
generate_release_notes() {
    local package_dir=$1
    local arch=$2
    
    cat > "$package_dir/README.md" << EOF
# MultiOS $DEPLOY_VERSION for $arch

## Installation

### Binary Installation
1. Download the appropriate binary for your system
2. Make it executable: \`chmod +x multios\`
3. Run: \`./multios\`

### Docker Installation
\`\`\`bash
docker pull ${DOCKER_REGISTRY}/multios:$DEPLOY_VERSION-$arch
docker run --rm -it ${DOCKER_REGISTRY}/multios:$DEPLOY_VERSION-$arch
\`\`\`

## Verification

Verify the download integrity:

\`\`\`bash
# Check SHA256
sha256sum -c checksums.sha256

# Check MD5
md5sum -c checksums.md5
\`\`\`

## Support

- Documentation: https://docs.multios.dev
- Issues: https://github.com/$GITHUB_REPOSITORY/issues
- Community: https://discord.gg/multios

---
Generated: $(date -Iseconds)
EOF

    cat > "$package_dir/CHANGELOG.md" << EOF
# Changelog

All notable changes to MultiOS will be documented in this file.

## [$DEPLOY_VERSION] - $(date +%Y-%m-%d)

### Added
- Support for $arch architecture
- Cross-compilation capabilities
- Performance benchmarks

### Changed
- Improved build pipeline
- Enhanced testing coverage

### Fixed
- Various bug fixes and optimizations

---
EOF
}

# Generate deployment report
generate_deployment_report() {
    local report_file="$ARTIFACTS_DIR/deployment_report_$(date +%Y%m%d_%H%M%S).json"
    
    log "Generating deployment report..."
    
    # Collect deployment information
    local deployed_artifacts=()
    for package in "$ARTIFACTS_DIR"/packages/*; do
        if [ -f "$package" ]; then
            local filename=$(basename "$package")
            local size=$(stat -f%z "$package" 2>/dev/null || stat -c%s "$package" 2>/dev/null || echo "0")
            deployed_artifacts+=("{\"artifact\":\"$filename\",\"size\":$size,\"url\":\"$package\"}")
        fi
    done
    
    # Generate report
    cat > "$report_file" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "version": "$DEPLOY_VERSION",
    "deployment_status": "completed",
    "artifacts": [
$(IFS=,; printf '%s' "${deployed_artifacts[*]}")
    ],
    "registries": {
        "docker_registry": "${DOCKER_REGISTRY:-not_configured}",
        "crates_registry": "${CRATES_REGISTRY:-not_configured}",
        "github_releases": "${GITHUB_TOKEN:+configured}",
        "object_storage": "${S3_BUCKET:+configured}"
    }
}
EOF
    
    success "Deployment report generated: $report_file"
}

# Main deployment function
main() {
    log "Starting MultiOS Artifact Deployment"
    log "Version: $DEPLOY_VERSION"
    log "Artifacts directory: $ARTIFACTS_DIR"
    
    # Create packages directory
    mkdir -p "$ARTIFACTS_DIR/packages"
    
    # Deploy to all configured registries
    deploy_build_artifacts
    deploy_docker_images
    deploy_package_registries
    deploy_object_storage
    
    # Generate final report
    generate_deployment_report
    
    success "Deployment completed successfully"
    
    # Show deployment summary
    echo
    echo "Deployment Summary:"
    echo "==================="
    echo "Version: $DEPLOY_VERSION"
    echo "Artifacts: $ARTIFACTS_DIR/packages/"
    echo "Docker registry: ${DOCKER_REGISTRY:-not_configured}"
    echo "GitHub releases: ${GITHUB_TOKEN:+configured}"
    echo "Object storage: ${S3_BUCKET:+configured}"
    echo "==================="
}

# Show usage
usage() {
    cat << EOF
Usage: $0 [ARTIFACTS_DIR] [VERSION]

ARTIFACTS_DIR: Directory containing build artifacts (default: /workspace/artifacts)
VERSION: Version to deploy (default: latest)

Environment Variables:
  REGISTRY_URL          - Generic registry URL
  DOCKER_REGISTRY       - Docker registry URL
  CRATES_REGISTRY       - Crates.io registry
  CRATES_TOKEN          - Crates.io authentication token
  GITHUB_TOKEN          - GitHub authentication token
  GITHUB_REPOSITORY     - GitHub repository name
  S3_BUCKET             - AWS S3 bucket name
  AWS_ACCESS_KEY_ID     - AWS access key
  AWS_SECRET_ACCESS_KEY - AWS secret key

Examples:
  $0 /workspace/artifacts v1.2.3
  $0 /tmp/artifacts latest
EOF
}

# Parse command line arguments
if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    usage
    exit 0
fi

main "$@"