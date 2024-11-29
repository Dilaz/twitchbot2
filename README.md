# TwitchBot2

TwitchBot2 is a Twitch bot built using Rust. It connects to Twitch, manages channels, handles messages and bans ad bots

## Features

- Connects to Twitch and Twitch Helix
- Manages channels and users
- Handles messages and bans users with banned words
- Loads and manages URLs
- Uses SeaORM for database interactions

## Getting Started

### Prerequisites

- Rust
- Docker (for PostgreSQL)

### Installation

1. Clone the repository:
    ```sh
    git clone https://github.com/dilaz/twitchbot2.git
    cd twitchbot2
    ```

2. Set up the environment variables:
    ```sh
    cp .env.example .env
    ```

3. Build the project:
    ```sh
    cargo build
    ```

### Running the Bot

1. Start the PostgreSQL database using Docker:
    ```sh
    docker-compose up -d
    ```

2. Run the migrations: See `migrations/README.md`

3. Start the bot:
    ```sh
    cargo run
    ```

## License

This project is licensed under the MIT License.