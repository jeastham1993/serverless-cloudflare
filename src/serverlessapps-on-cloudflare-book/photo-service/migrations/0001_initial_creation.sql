-- Migration number: 0001 	 2024-05-21T10:17:19.368Z
CREATE TABLE image_categories (
                                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                                  slug TEXT UNIQUE,display_name TEXT
                                created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO image_categories (slug, display_name) VALUES
                                                      ('animals', 'Animals'),
                                                      ('landscapes', 'Landscapes'),
                                                      ('sports-cars', 'Sports Cars');
CREATE TABLE images (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        category_id INTEGER NOT NULL,
                        user_id INTEGER NOT NULL,
                        image_url TEXT NOT NULL,
                        title TEXT NOT NULL,
                        format TEXT NOT NULL,
                        resolution TEXT NOT NULL,
                        file_size_bytes INTEGER NOT NULL,
                        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                        FOREIGN KEY (category_id) REFERENCES image_categories(id)
);
CREATE INDEX IF NOT EXISTS idx_images_created_at ON images(created_at);
INSERT INTO images
(category_id, user_id, image_url, title,
 format, resolution, file_size_bytes)
VALUES
    (1, 1, 'https://example.com/some_image.png',
     'Example 1', 'PNG', '600x400', 1024),
    (2, 2, 'https://example.com/another_image.jpg',
     'Example 2', 'JPG', '600x400', 1024),
    (2, 3, 'https://example.com/one_more_image.png',
     'Example 3', 'PNG', '600x400', 1024),
    (3, 4, 'https://example.com/last_mage.jpg',
     'Example 4', 'JPG', '600x400', 1024);