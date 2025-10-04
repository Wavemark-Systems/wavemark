#!/bin/bash

# Setup script for GitHub secrets
# This script helps you set up the required secrets for PyPI publishing

echo "🔧 Setting up GitHub secrets for PyPI publishing..."
echo ""

# Check if .env file exists
if [ ! -f ".env" ]; then
    echo "❌ .env file not found!"
    echo "Please create a .env file with your PyPI credentials:"
    echo "USERNAME=\"__token__\""
    echo "PASSWORD=\"your_pypi_token_here\""
    exit 1
fi

# Load environment variables
source .env

echo "📋 GitHub Secrets to add:"
echo ""
echo "Go to: https://github.com/YOUR_USERNAME/wavemark/settings/secrets/actions"
echo ""
echo "Add these secrets:"
echo ""

if [ -n "$USERNAME" ]; then
    echo "🔑 PYPI_USERNAME"
    echo "   Value: $USERNAME"
    echo ""
fi

if [ -n "$PASSWORD" ]; then
    echo "🔑 PYPI_PASSWORD" 
    echo "   Value: $PASSWORD"
    echo ""
fi

# Also suggest the token approach
echo "🔑 PYPI_API_TOKEN (alternative to username/password)"
echo "   Value: $PASSWORD"
echo ""

echo "📝 Instructions:"
echo "1. Go to your GitHub repository settings"
echo "2. Navigate to Secrets and variables > Actions"
echo "3. Click 'New repository secret'"
echo "4. Add each secret with the name and value shown above"
echo ""
echo "🚀 After adding secrets, you can:"
echo "   - Run 'Publish to PyPI Now' workflow manually"
echo "   - Create a GitHub release to trigger automatic publishing"
echo "   - Push to main branch to run tests"
echo ""
echo "✅ Setup complete!"
