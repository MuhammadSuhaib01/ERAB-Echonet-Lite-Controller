# Build stage
FROM node:22-alpine AS builder

WORKDIR /app

# Copy package files
COPY ./ssng-node2/package*.json ./

# Install all dependencies (including dev)
RUN npm ci

# Copy application files
COPY ./ssng-node2 .

# Build the frontend
RUN npm run build

# Production stage
FROM node:22-alpine

WORKDIR /app

# Copy package files
COPY ./ssng-node2/package*.json ./

# Install only production dependencies
RUN npm ci --omit=dev

# Copy application files from builder
COPY --from=builder /app/dist ./dist

# Copy other necessary files
COPY ./ssng-node2/app.js ./
COPY ./ssng-node2/index.html ./
COPY ./ssng-node2/public ./public

# Expose ports
EXPOSE 3000 8080

# Start the application
CMD ["npm", "start"]