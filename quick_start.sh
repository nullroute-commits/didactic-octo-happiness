#!/bin/bash
# Quick start script for Automation Nation

set -e

echo "🚀 Automation Nation Quick Start"
echo "================================"

# Function to detect the correct Docker Compose command
detect_docker_compose() {
    if command -v docker-compose >/dev/null 2>&1; then
        echo "docker-compose"
    elif command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
        echo "docker compose"
    else
        echo ""
    fi
}

# Get the correct Docker Compose command
DOCKER_COMPOSE_CMD=$(detect_docker_compose)

# Check if .env exists, if not copy from template
if [ ! -f .env ]; then
    echo "📝 Creating .env file from template..."
    cp .env.template .env
    echo "⚠️  Please edit .env file to set secure passwords!"
    echo "   You can continue with defaults for testing."
    read -p "Press Enter to continue with default passwords (not recommended for production)..."
fi

# Choose deployment method
echo ""
echo "📦 Choose deployment method:"
echo "1) Docker Compose (full stack with monitoring)"
echo "2) Docker (web app only)"
echo "3) Podman (web app only)"
echo "4) LXC (requires root)"
echo ""
read -p "Enter choice (1-4): " choice

case $choice in
    1)
        echo "🐳 Starting with Docker Compose..."
        
        # Check if Docker Compose is available
        if [[ -z "$DOCKER_COMPOSE_CMD" ]]; then
            echo "❌ Docker and Docker Compose are required"
            echo "   Please install Docker first: https://docs.docker.com/get-docker/"
            exit 1
        fi
        
        echo "📋 Using Docker Compose command: $DOCKER_COMPOSE_CMD"
        
        # Build and start services
        echo "🔨 Building application..."
        $DOCKER_COMPOSE_CMD build automation-nation-web
        
        echo "🚀 Starting all services..."
        $DOCKER_COMPOSE_CMD up -d
        
        echo "⏳ Waiting for services to be ready..."
        sleep 30
        
        echo "✅ Services started!"
        echo ""
        echo "🌐 Access URLs:"
        echo "   Web Application: http://localhost:3000"
        echo "   NetBox:         http://localhost:8080"
        echo "   Prometheus:     http://localhost:9090"
        echo "   Grafana:        http://localhost:3001"
        echo "   Kibana:         http://localhost:5601"
        echo ""
        echo "📊 Default credentials:"
        echo "   NetBox:  admin / (check .env file)"
        echo "   Grafana: admin / (check .env file)"
        ;;
        
    2)
        echo "🐳 Starting with Docker..."
        
        if ! command -v docker &> /dev/null; then
            echo "❌ Docker is required"
            exit 1
        fi
        
        echo "🔨 Building Docker image..."
        docker build -t automation-nation .
        
        echo "🚀 Starting container..."
        docker run -d \
            --name automation-nation \
            -p 3000:3000 \
            -v /proc:/host/proc:ro \
            -v /sys:/host/sys:ro \
            -v /var/run/docker.sock:/var/run/docker.sock:rw \
            --env-file .env \
            automation-nation
        
        echo "✅ Container started!"
        echo "🌐 Access: http://localhost:3000"
        ;;
        
    3)
        echo "🦭 Starting with Podman..."
        
        if ! command -v podman &> /dev/null; then
            echo "❌ Podman is required"
            exit 1
        fi
        
        echo "🔨 Building Podman image..."
        podman build -t automation-nation -f Containerfile .
        
        echo "🚀 Starting container..."
        podman run -d \
            --name automation-nation \
            -p 3000:3000 \
            -v /proc:/host/proc:ro \
            -v /sys:/host/sys:ro \
            -v /run/podman/podman.sock:/run/podman/podman.sock:rw \
            --env-file .env \
            automation-nation
        
        echo "✅ Container started!"
        echo "🌐 Access: http://localhost:3000"
        ;;
        
    4)
        echo "📦 Setting up LXC container..."
        
        if [ "$EUID" -ne 0 ]; then
            echo "❌ LXC setup requires root privileges"
            echo "Run: sudo $0"
            exit 1
        fi
        
        if ! command -v lxc-create &> /dev/null; then
            echo "❌ LXC is required"
            exit 1
        fi
        
        echo "🔨 Running LXC setup..."
        ./setup_lxc.sh
        
        echo "✅ LXC container setup complete!"
        echo "🌐 Access: http://localhost:3000"
        ;;
        
    *)
        echo "❌ Invalid choice"
        exit 1
        ;;
esac

echo ""
echo "🎉 Automation Nation is starting up!"
echo ""
echo "📖 Next steps:"
echo "   1. Wait a few moments for all services to initialize"
echo "   2. Access the web interface at http://localhost:3000"
echo "   3. Run system information collection"
echo "   4. Deploy containers using the web interface"
echo ""
echo "🔍 To check status:"
if [ "$choice" = "1" ]; then
    echo "   $DOCKER_COMPOSE_CMD ps"
    echo "   $DOCKER_COMPOSE_CMD logs -f automation-nation-web"
elif [ "$choice" = "2" ]; then
    echo "   docker ps"
    echo "   docker logs -f automation-nation"
elif [ "$choice" = "3" ]; then
    echo "   podman ps"
    echo "   podman logs -f automation-nation"
elif [ "$choice" = "4" ]; then
    echo "   sudo lxc-ls -f"
    echo "   sudo lxc-attach -n automation-nation"
fi
echo ""
echo "🛑 To stop:"
if [ "$choice" = "1" ]; then
    echo "   $DOCKER_COMPOSE_CMD down"
elif [ "$choice" = "2" ]; then
    echo "   docker stop automation-nation && docker rm automation-nation"
elif [ "$choice" = "3" ]; then
    echo "   podman stop automation-nation && podman rm automation-nation"
elif [ "$choice" = "4" ]; then
    echo "   sudo lxc-stop -n automation-nation"
fi