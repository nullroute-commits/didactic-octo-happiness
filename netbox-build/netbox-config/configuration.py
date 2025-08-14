# NetBox configuration for Automation Nation
import os
import sys

# Base configuration
BASE_DIR = os.path.dirname(os.path.abspath(__file__))

# Database configuration
DATABASE = {
    'NAME': os.environ.get('DB_NAME', 'netbox'),
    'USER': os.environ.get('DB_USER', 'netbox'),
    'PASSWORD': os.environ.get('DB_PASSWORD', ''),
    'HOST': os.environ.get('DB_HOST', 'localhost'),
    'PORT': os.environ.get('DB_PORT', ''),
    'CONN_MAX_AGE': int(os.environ.get('DB_CONN_MAX_AGE', '300')),
}

# Redis configuration
REDIS = {
    'tasks': {
        'HOST': os.environ.get('REDIS_HOST', 'localhost'),
        'PORT': int(os.environ.get('REDIS_PORT', '6379')),
        'PASSWORD': os.environ.get('REDIS_PASSWORD', ''),
        'DATABASE': int(os.environ.get('REDIS_DATABASE', '0')),
        'SSL': os.environ.get('REDIS_SSL', 'False').lower() == 'true',
    },
    'caching': {
        'HOST': os.environ.get('REDIS_CACHE_HOST', os.environ.get('REDIS_HOST', 'localhost')),
        'PORT': int(os.environ.get('REDIS_CACHE_PORT', os.environ.get('REDIS_PORT', '6379'))),
        'PASSWORD': os.environ.get('REDIS_CACHE_PASSWORD', os.environ.get('REDIS_PASSWORD', '')),
        'DATABASE': int(os.environ.get('REDIS_CACHE_DATABASE', '1')),
        'SSL': os.environ.get('REDIS_CACHE_SSL', os.environ.get('REDIS_SSL', 'False')).lower() == 'true',
    }
}

# Secret key for cryptographic signing
SECRET_KEY = os.environ.get('SECRET_KEY', 'netbox-secret-key-change-me-in-production')

# Allowed hosts
ALLOWED_HOSTS = os.environ.get('ALLOWED_HOSTS', '*').split(',')

# Debug mode
DEBUG = os.environ.get('DEBUG', 'False').lower() == 'true'

# Email configuration
EMAIL = {
    'SERVER': os.environ.get('EMAIL_SERVER', 'localhost'),
    'PORT': int(os.environ.get('EMAIL_PORT', '25')),
    'USERNAME': os.environ.get('EMAIL_USERNAME', ''),
    'PASSWORD': os.environ.get('EMAIL_PASSWORD', ''),
    'USE_SSL': os.environ.get('EMAIL_USE_SSL', 'False').lower() == 'true',
    'USE_TLS': os.environ.get('EMAIL_USE_TLS', 'False').lower() == 'true',
    'SSL_CERTFILE': os.environ.get('EMAIL_SSL_CERTFILE', ''),
    'SSL_KEYFILE': os.environ.get('EMAIL_SSL_KEYFILE', ''),
    'TIMEOUT': int(os.environ.get('EMAIL_TIMEOUT', '10')),
    'FROM': os.environ.get('EMAIL_FROM', 'netbox@automation-nation.local'),
}

# Time zone
TIME_ZONE = os.environ.get('TIME_ZONE', 'UTC')

# Date/time formatting
DATE_FORMAT = 'N j, Y'
SHORT_DATE_FORMAT = 'Y-m-d'
TIME_FORMAT = 'g:i a'
SHORT_TIME_FORMAT = 'H:i:s'
DATETIME_FORMAT = 'N j, Y g:i a'
SHORT_DATETIME_FORMAT = 'Y-m-d H:i'

# Pagination
PAGINATE_COUNT = int(os.environ.get('PAGINATE_COUNT', '50'))

# Enable installed plugins
try:
    from plugins_config import PLUGINS, PLUGINS_CONFIG
except ImportError:
    PLUGINS = []
    PLUGINS_CONFIG = {}

# Automation Nation specific configuration
CORS_ALLOW_ALL_ORIGINS = os.environ.get('CORS_ORIGIN_ALLOW_ALL', 'True').lower() == 'true'

# API configuration
REST_FRAMEWORK = {
    'DEFAULT_PAGINATION_CLASS': 'netbox.api.pagination.OptionalLimitOffsetPagination',
    'PAGE_SIZE': int(os.environ.get('API_PAGE_SIZE', '50')),
    'DEFAULT_RENDERER_CLASSES': [
        'rest_framework.renderers.JSONRenderer',
        'netbox.api.renderers.FormlessBrowsableAPIRenderer',
    ],
    'DEFAULT_PARSER_CLASSES': [
        'rest_framework.parsers.JSONParser',
        'rest_framework.parsers.MultiPartParser',
    ],
    'DEFAULT_AUTHENTICATION_CLASSES': [
        'rest_framework.authentication.SessionAuthentication',
        'netbox.api.authentication.TokenAuthentication',
    ],
    'DEFAULT_PERMISSION_CLASSES': [
        'rest_framework.permissions.IsAuthenticated',
    ],
    'DEFAULT_VERSIONING_CLASS': 'rest_framework.versioning.AcceptHeaderVersioning',
    'DEFAULT_VERSION': '2.8',
    'ALLOWED_VERSIONS': ['2.8'],
    'VERSION_PARAM': 'version',
}

# Security settings
SECURE_PROXY_SSL_HEADER = ('HTTP_X_FORWARDED_PROTO', 'https')
USE_X_FORWARDED_HOST = True
USE_X_FORWARDED_PORT = True

# Logging configuration
LOGGING = {
    'version': 1,
    'disable_existing_loggers': False,
    'formatters': {
        'verbose': {
            'format': '{levelname} {asctime} {module} {process:d} {thread:d} {message}',
            'style': '{',
        },
        'simple': {
            'format': '{levelname} {message}',
            'style': '{',
        },
    },
    'filters': {
        'require_debug_false': {
            '()': 'django.utils.log.RequireDebugFalse',
        },
    },
    'handlers': {
        'console': {
            'level': 'INFO',
            'class': 'logging.StreamHandler',
            'formatter': 'simple'
        },
        'file': {
            'level': 'WARNING',
            'class': 'logging.handlers.RotatingFileHandler',
            'filename': '/opt/netbox/netbox/logs/netbox.log',
            'maxBytes': 1024*1024*10,  # 10 MB
            'backupCount': 5,
            'formatter': 'verbose',
        },
    },
    'loggers': {
        'django': {
            'handlers': ['console', 'file'],
            'level': os.getenv('DJANGO_LOG_LEVEL', 'INFO'),
        },
        'netbox': {
            'handlers': ['console', 'file'],
            'level': os.getenv('NETBOX_LOG_LEVEL', 'INFO'),
        },
    },
}

# Create logs directory if it doesn't exist
import os
log_dir = '/opt/netbox/netbox/logs'
if not os.path.exists(log_dir):
    os.makedirs(log_dir, exist_ok=True)

# Additional Django settings
DEFAULT_AUTO_FIELD = 'django.db.models.AutoField'