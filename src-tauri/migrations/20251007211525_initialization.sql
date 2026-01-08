CREATE TABLE app_users (
    user_id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT DEFAULT 'user',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE connection_folders (
    folder_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    folder_name TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES app_users(user_id) ON DELETE CASCADE
);

CREATE TABLE connections (
    connection_id INTEGER PRIMARY KEY AUTOINCREMENT,
    folder_id INTEGER,
    user_id INTEGER NOT NULL,
    connection_name TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER DEFAULT 5432,
    db_name TEXT NOT NULL,
    db_user TEXT NOT NULL,
    db_password_encrypted TEXT NOT NULL,
    ssl_mode TEXT DEFAULT 'prefer',
    FOREIGN KEY (folder_id) REFERENCES connection_folders(folder_id) ON DELETE SET NULL,
    FOREIGN KEY (user_id) REFERENCES app_users(user_id) ON DELETE CASCADE
);

CREATE TABLE tags (
    tag_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    tag_name TEXT NOT NULL,
    color_hex TEXT DEFAULT '#FFFFFF',
    FOREIGN KEY (user_id) REFERENCES app_users(user_id) ON DELETE CASCADE
);

CREATE TABLE connection_tags (
    tag_id INTEGER NOT NULL,
    connection_id INTEGER NOT NULL,
    PRIMARY KEY (tag_id, connection_id),
    FOREIGN KEY (tag_id) REFERENCES tags(tag_id) ON DELETE CASCADE,
    FOREIGN KEY (connection_id) REFERENCES connections(connection_id) ON DELETE CASCADE
);

CREATE TABLE query_history (
    history_id INTEGER PRIMARY KEY AUTOINCREMENT,
    connection_id INTEGER NOT NULL,
    query_text TEXT NOT NULL,
    status TEXT NOT NULL, -- 'success', 'error'
    execution_time_ms INTEGER,
    error_message TEXT,
    executed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (connection_id) REFERENCES connections(connection_id) ON DELETE CASCADE
);

CREATE TABLE pinned_queries (
    pinned_query_id INTEGER PRIMARY KEY AUTOINCREMENT,
    connection_id INTEGER NOT NULL,
    query_name TEXT NOT NULL,
    query_text TEXT NOT NULL,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (connection_id) REFERENCES connections(connection_id) ON DELETE CASCADE
);

CREATE TABLE bookmarks (
    bookmark_id INTEGER PRIMARY KEY AUTOINCREMENT,
    connection_id INTEGER NOT NULL,
    schema_name TEXT NOT NULL,
    object_name TEXT NOT NULL,
    object_type TEXT NOT NULL,
    FOREIGN KEY (connection_id) REFERENCES connections(connection_id) ON DELETE CASCADE
);

CREATE TABLE diagrams (
    diagram_id INTEGER PRIMARY KEY AUTOINCREMENT,
    connection_id INTEGER NOT NULL,
    diagram_name TEXT NOT NULL,
    definition_json TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (connection_id) REFERENCES connections(connection_id) ON DELETE CASCADE
);

CREATE TABLE app_user_logs (
    log_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    action_type TEXT NOT NULL,
    details TEXT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES app_users(user_id) ON DELETE CASCADE
);