# Use the official PostgreSQL image from the Docker Hub
FROM postgres:latest

# Set environment variables for PostgreSQL user, password, and database
ENV POSTGRES_USER=postgres
ENV POSTGRES_PASSWORD=123456
ENV POSTGRES_DB=uchat
ENV POSTGRES_DB_TEST=uchat_test

# Expose the PostgreSQL port
EXPOSE 5432

# Optionally, you can add any custom initialization scripts in /docker-entrypoint-initdb.d/
# COPY init.sql /docker-entrypoint-initdb.d/

# Default command to run PostgreSQL server
CMD ["postgres"]
