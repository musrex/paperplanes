CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    uuid UUID NOT NULL,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    role TEXT DEFAULT 'user',
    reset_token TEXT,
    reset_token_expires TIMESTAMP,
    profile_text TEXT
);

CREATE TABLE groups (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE group_members (
    group_id INT REFERENCES groups(id) ON DELETE CASCADE,
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (group_id, user_id)
);

CREATE TABLE messages (
    id SERIAL PRIMARY KEY,
    from_user INT REFERENCES users(id) ON DELETE CASCADE,
    to_user INT REFERENCES users(id) ON DELETE CASCADE,
    group_id INT REFERENCES groups(id),
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    expiration TIMESTAMP NOT NULL
 );
