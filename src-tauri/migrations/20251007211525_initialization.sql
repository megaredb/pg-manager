CREATE TABLE app_users (
    user_id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE connection_folders (
    connections_folder_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    folder_name TEXT NOT NULL,
    
    FOREIGN KEY(user_id) 
        REFERENCES app_users(user_id)
        ON DELETE CASCADE
);

CREATE TABLE connections (
    connection_id INTEGER PRIMARY KEY AUTOINCREMENT,
    connections_folder_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    connection_name TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER NOT NULL,
    db_name TEXT NOT NULL,
    db_user TEXT NOT NULL,
    db_password_encrypted TEXT,
    ssl_mode TEXT DEFAULT 'prefer',

    FOREIGN KEY(connections_folder_id) 
        REFERENCES connection_folders(connections_folder_id) 
        ON DELETE CASCADE,

    FOREIGN KEY(user_id) 
        REFERENCES app_users(user_id) 
        ON DELETE CASCADE
);

INSERT INTO app_users (username, password_hash, role) VALUES
('admin', 'hash_admin', 'admin'),
('john_doe', 'hash_john', 'user'),
('jane_smith', 'hash_jane', 'user'),
('mike_b', 'hash_mike', 'user'),
('sara_l', 'hash_sara', 'user');

INSERT INTO connection_folders (user_id, folder_name) VALUES
(1, 'Admin Prod DBs'),
(1, 'Admin Dev DBs'),
(2, 'Johns Work Projects'),
(2, 'Johns Personal'),
(3, 'Janes Analytics DBs'),
(5, 'Saras Sandbox');

INSERT INTO connections (connections_folder_id, user_id, connection_name, host, port, db_name, db_user, db_password_encrypted) VALUES
(1, 1, 'Prod Main', 'prod.server.com', 5432, 'main_db', 'admin_prod', 'enc_pass_1'),
(1, 1, 'Prod Analytics', 'analytics.server.com', 5432, 'analytics_db', 'admin_prod', 'enc_pass_2'),
(2, 1, 'Dev Main', 'dev.local', 5432, 'main_db_dev', 'dev_user', 'enc_pass_3'),
(3, 2, 'Project Phoenix', '192.168.1.10', 5433, 'phoenix', 'john_p', 'enc_pass_4'),
(3, 2, 'Project Delta', '192.168.1.11', 5432, 'delta', 'john_d', 'enc_pass_5'),
(5, 3, 'Data Warehouse', 'dw.company.com', 5439, 'sales_dw', 'jane_a', 'enc_pass_6'),
(6, 5, 'Local Test', 'localhost', 5432, 'test_db', 'sara_dev', 'enc_pass_7');