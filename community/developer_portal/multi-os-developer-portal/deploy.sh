#!/bin/bash

# MultiOS Developer Portal Deployment Script
# Supports deployment to Vercel, Netlify, Railway, and Docker

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Help function
show_help() {
    echo "MultiOS Developer Portal Deployment Script"
    echo ""
    echo "Usage: $0 [OPTIONS] TARGET"
    echo ""
    echo "TARGET:"
    echo "  vercel      Deploy to Vercel"
    echo "  netlify     Deploy to Netlify"
    echo "  railway     Deploy to Railway"
    echo "  docker      Build Docker images"
    echo "  build       Build for production"
    echo ""
    echo "OPTIONS:"
    echo "  -h, --help     Show this help message"
    echo "  -d, --dev      Use development environment"
    echo "  -p, --prod     Use production environment"
    echo ""
    echo "Examples:"
    echo "  $0 vercel -d    # Deploy to Vercel in development mode"
    echo "  $0 docker -p    # Build production Docker images"
}

# Parse arguments
ENV="production"
TARGET=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -d|--dev)
            ENV="development"
            shift
            ;;
        -p|--prod)
            ENV="production"
            shift
            ;;
        vercel|netlify|railway|docker|build)
            TARGET="$1"
            shift
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

if [ -z "$TARGET" ]; then
    echo -e "${RED}Error: No deployment target specified${NC}"
    show_help
    exit 1
fi

# Check prerequisites
check_prerequisites() {
    case $TARGET in
        vercel)
            if ! command -v vercel &> /dev/null; then
                echo -e "${RED}Error: Vercel CLI not installed. Install with: npm install -g vercel${NC}"
                exit 1
            fi
            ;;
        netlify)
            if ! command -v netlify &> /dev/null; then
                echo -e "${RED}Error: Netlify CLI not installed. Install with: npm install -g netlify-cli${NC}"
                exit 1
            fi
            ;;
        railway)
            if ! command -v railway &> /dev/null; then
                echo -e "${RED}Error: Railway CLI not installed. Install with: npm install -g @railway/cli${NC}"
                exit 1
            fi
            ;;
        docker)
            if ! command -v docker &> /dev/null; then
                echo -e "${RED}Error: Docker not installed${NC}"
                exit 1
            fi
            ;;
    esac
}

# Build the application
build_app() {
    echo -e "${BLUE}Building application for $ENV environment...${NC}"
    
    # Install dependencies
    pnpm install
    
    # Install backend dependencies
    cd server && pnpm install && cd ..
    
    # Build frontend
    pnpm build
    
    echo -e "${GREEN}Build completed successfully${NC}"
}

# Deploy to Vercel
deploy_vercel() {
    echo -e "${BLUE}Deploying to Vercel...${NC}"
    
    # Build first
    if [ "$ENV" = "production" ]; then
        pnpm build:prod
    else
        pnpm build
    fi
    
    # Deploy
    if [ "$ENV" = "production" ]; then
        vercel --prod
    else
        vercel
    fi
}

# Deploy to Netlify
deploy_netlify() {
    echo -e "${BLUE}Deploying to Netlify...${NC}"
    
    # Build first
    if [ "$ENV" = "production" ]; then
        pnpm build:prod
    else
        pnpm build
    fi
    
    # Deploy
    if [ "$ENV" = "production" ]; then
        netlify deploy --prod --dir=dist
    else
        netlify deploy --dir=dist
    fi
}

# Deploy to Railway
deploy_railway() {
    echo -e "${BLUE}Deploying to Railway...${NC}"
    
    # Login to Railway
    railway login
    
    # Deploy backend
    cd server
    railway deploy
    
    # Set environment variables
    if [ "$ENV" = "production" ]; then
        railway variables set NODE_ENV=production
    else
        railway variables set NODE_ENV=development
    fi
    
    cd ..
    
    echo -e "${GREEN}Backend deployed to Railway${NC}"
    echo -e "${YELLOW}Note: Deploy frontend separately to Vercel or Netlify${NC}"
}

# Build Docker images
build_docker() {
    echo -e "${BLUE}Building Docker images...${NC}"
    
    # Build frontend
    docker build -t multios-frontend:latest .
    
    # Build backend
    docker build -t multios-backend:latest server/
    
    # Build with specific tag for environment
    if [ "$ENV" = "production" ]; then
        docker tag multios-frontend:latest multios-frontend:prod
        docker tag multios-backend:latest multios-backend:prod
    else
        docker tag multios-frontend:latest multios-frontend:dev
        docker tag multios-backend:latest multios-backend:dev
    fi
    
    echo -e "${GREEN}Docker images built successfully${NC}"
    echo -e "${BLUE}Images:${NC}"
    echo "  multios-frontend:latest ($ENV)"
    echo "  multios-backend:latest ($ENV)"
    
    # Show image sizes
    echo -e "\n${BLUE}Image sizes:${NC}"
    docker images | grep multios
}

# Main deployment logic
main() {
    echo -e "${GREEN}🚀 Starting deployment to $TARGET ($ENV environment)${NC}"
    echo ""
    
    check_prerequisites
    
    case $TARGET in
        vercel)
            deploy_vercel
            ;;
        netlify)
            deploy_netlify
            ;;
        railway)
            deploy_railway
            ;;
        docker)
            build_docker
            ;;
        build)
            build_app
            ;;
        *)
            echo -e "${RED}Error: Unknown target '$TARGET'${NC}"
            show_help
            exit 1
            ;;
    esac
    
    echo -e "\n${GREEN}✅ Deployment completed successfully!${NC}"
}

# Run main function
main