#!/bin/bash

# Moonshot Indexer Setup Script
# This script helps set up the initial environment for the indexer

set -e

echo "🚀 Setting up Moonshot Indexer..."

# Check if .env file exists
if [ ! -f .env ]; then
    echo "📝 Creating .env file..."
    cat > .env << EOF
# Abstract Chain Configuration
RPC_URL=wss://rpc.abstract.money
DATABASE_URL=postgresql://postgres:password@localhost:5432/moonshot_indexer
LOG_LEVEL=info
CHAIN_ID=8453
MOONSHOT_FACTORY_ADDRESS=0x0000000000000000000000000000000000000000
BATCH_SIZE=100
POLL_INTERVAL_MS=1000
EOF
    echo "✅ .env file created"
else
    echo "✅ .env file already exists"
fi

# Check if Docker is available
if command -v docker &> /dev/null; then
    echo "🐳 Docker detected. Would you like to set up PostgreSQL with Docker? (y/n)"
    read -r response
    if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
        echo "📦 Setting up PostgreSQL with Docker..."
        
        # Check if container already exists
        if docker ps -a --format 'table {{.Names}}' | grep -q "postgres-moonshot"; then
            echo "🔄 PostgreSQL container already exists. Starting it..."
            docker start postgres-moonshot
        else
            echo "🆕 Creating new PostgreSQL container..."
            docker run --name postgres-moonshot \
                -e POSTGRES_PASSWORD=password \
                -e POSTGRES_DB=moonshot_indexer \
                -p 5432:5432 \
                -d postgres:17
            
            echo "⏳ Waiting for PostgreSQL to start..."
            sleep 10
        fi
        
        echo "✅ PostgreSQL is running on localhost:5432"
        echo "   Database: moonshot_indexer"
        echo "   Username: postgres"
        echo "   Password: password"
    fi
else
    echo "⚠️  Docker not found. Please install PostgreSQL manually:"
    echo "   - Install PostgreSQL 12+"
    echo "   - Create database: moonshot_indexer"
    echo "   - Update DATABASE_URL in .env file"
fi

# Check if Rust is installed
if command -v cargo &> /dev/null; then
    echo "🦀 Rust detected. Building project..."
    cargo build
    echo "✅ Build completed"
else
    echo "⚠️  Rust not found. Please install Rust:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
fi

echo ""
echo "🎉 Setup complete!"
echo ""
echo "Next steps:"
echo "1. Update the .env file with your actual configuration:"
echo "   - Set the correct Abstract chain RPC URL"
echo "   - Set the actual Moonshot factory address"
echo "   - Update database connection if needed"
echo ""
echo "2. Run the indexer:"
echo "   cargo run"
echo ""
echo "3. Check the logs to ensure everything is working"
echo ""
echo "📚 For more information, see README.md and demo.md" 