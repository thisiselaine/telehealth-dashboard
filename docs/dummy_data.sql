-- Insert dummy users
INSERT INTO users (id, username, email, password) VALUES
(1, 'test_user1', 'test1@example.com', 'hashed_password1'),
(2, 'test_user2', 'test2@example.com', 'hashed_password2');

-- Insert dummy favorites
INSERT INTO favorites (id, user_id, location_id, practitioner_name) VALUES
(1, 1, '12345', 'Dr. Smith'),
(2, 2, '67890', 'Dr. Johnson');
