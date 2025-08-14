#!/bin/bash
# NetBox Docker entrypoint script

set -e

echo "Starting NetBox container..."

# Wait for database to be ready
echo "Waiting for database to be ready..."
while ! nc -z ${DB_HOST:-postgres} ${DB_PORT:-5432}; do
    echo "Database not ready, waiting..."
    sleep 2
done

echo "Database is ready"

# Wait for Redis to be ready
echo "Waiting for Redis to be ready..."
while ! nc -z ${REDIS_HOST:-redis} ${REDIS_PORT:-6379}; do
    echo "Redis not ready, waiting..."
    sleep 2
done

echo "Redis is ready"

# Run database migrations
echo "Running database migrations..."
python manage.py migrate --no-input

# Collect static files
echo "Collecting static files..."
python manage.py collectstatic --no-input --clear

# Create superuser if specified and doesn't exist
if [ -n "$SUPERUSER_NAME" ] && [ -n "$SUPERUSER_EMAIL" ] && [ -n "$SUPERUSER_PASSWORD" ]; then
    echo "Checking for superuser..."
    python manage.py shell << EOF
from django.contrib.auth.models import User
if not User.objects.filter(username='$SUPERUSER_NAME').exists():
    User.objects.create_superuser('$SUPERUSER_NAME', '$SUPERUSER_EMAIL', '$SUPERUSER_PASSWORD')
    print('Superuser created successfully')
else:
    print('Superuser already exists')
EOF
fi

# Load initial data if specified
if [ -n "$LOAD_INITIAL_DATA" ] && [ "$LOAD_INITIAL_DATA" = "true" ]; then
    echo "Loading initial data..."
    python manage.py loaddata initial_data
fi

# Clear any stale cache
echo "Clearing cache..."
python manage.py invalidate all

echo "NetBox initialization completed"

# Execute the main command
exec "$@"