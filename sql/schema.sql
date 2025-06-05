DROP SCHEMA IF EXISTS public CASCADE;
CREATE SCHEMA public;

CREATE TABLE public.posts (
  id               SERIAL           PRIMARY KEY,
  title            VARCHAR(255)     NOT NULL,
  description      TEXT             NOT NULL DEFAULT '',
  body             TEXT             NOT NULL,
  tags             TEXT[]           NOT NULL DEFAULT '{}',
  thumbnail        TEXT             NOT NULL DEFAULT '',
  thumbnail_blur   TEXT             NOT NULL DEFAULT '/placeholder_image.png',
  created_at       TIMESTAMP        NOT NULL DEFAULT NOW()
);

INSERT INTO public.posts (title, description, body, tags, thumbnail, thumbnail_blur) VALUES
('첫 번째 게시물', '샘플 설명 1', '샘플 본문 내용 1', '{"rust","actix"}', '/placeholder_image.png', '/placeholder_image.png'),
('두 번째 게시물', '샘플 설명 2', '샘플 본문 내용 2', '{"postgres","sql"}', '/placeholder_image.png', '/placeholder_image.png'),
('세 번째 게시물', '샘플 설명 3', '샘플 본문 내용 3', '{"api","backend"}', '/placeholder_image.png', '/placeholder_image.png'),
('네 번째 게시물', '샘플 설명 4', '샘플 본문 내용 4', '{"docker","compose"}', '/placeholder_image.png', '/placeholder_image.png'),
('다섯 번째 게시물', '샘플 설명 5', '샘플 본문 내용 5', '{"testing","rust"}', '/placeholder_image.png', '/placeholder_image.png'),
('여섯 번째 게시물', '샘플 설명 6', '샘플 본문 내용 6', '{"actix","http"}', '/placeholder_image.png', '/placeholder_image.png'),
('일곱 번째 게시물', '샘플 설명 7', '샘플 본문 내용 7', '{"web","service"}', '/placeholder_image.png', '/placeholder_image.png'),
('여덟 번째 게시물', '샘플 설명 8', '샘플 본문 내용 8', '{"query","database"}', '/placeholder_image.png', '/placeholder_image.png'),
('아홉 번째 게시물', '샘플 설명 9', '샘플 본문 내용 9', '{"pagination","limit"}', '/placeholder_image.png', '/placeholder_image.png'),
('열 번째 게시물', '샘플 설명 10', '샘플 본문 내용 10', '{"offset","order"}', '/placeholder_image.png', '/placeholder_image.png'),
('열한 번째 게시물', '샘플 설명 11', '샘플 본문 내용 11', '{"json","serde"}', '/placeholder_image.png', '/placeholder_image.png'),
('열두 번째 게시물', '샘플 설명 12', '샘플 본문 내용 12', '{"error","handling"}', '/placeholder_image.png', '/placeholder_image.png'),
('열세 번째 게시물', '샘플 설명 13', '샘플 본문 내용 13', '{"pagination","actix"}', '/placeholder_image.png', '/placeholder_image.png'),
('열네 번째 게시물', '샘플 설명 14', '샘플 본문 내용 14', '{"middleware","rust"}', '/placeholder_image.png', '/placeholder_image.png'),
('열다섯 번째 게시물', '샘플 설명 15', '샘플 본문 내용 15', '{"dotenv","env"}', '/placeholder_image.png', '/placeholder_image.png'),
('열여섯 번째 게시물', '샘플 설명 16', '샘플 본문 내용 16', '{"configuration","confik"}', '/placeholder_image.png', '/placeholder_image.png'),
('열일곱 번째 게시물', '샘플 설명 17', '샘플 본문 내용 17', '{"logging","debug"}', '/placeholder_image.png', '/placeholder_image.png'),
('열여덟 번째 게시물', '샘플 설명 18', '샘플 본문 내용 18', '{"sqlx","query"}', '/placeholder_image.png', '/placeholder_image.png'),
('열아홉 번째 게시물', '샘플 설명 19', '샘플 본문 내용 19', '{"array","type"}', '/placeholder_image.png', '/placeholder_image.png'),
('스무 번째 게시물', '샘플 설명 20', '샘플 본문 내용 20', '{"structure","design"}', '/placeholder_image.png', '/placeholder_image.png');
