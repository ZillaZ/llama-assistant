CREATE TABLE Context(
  chat_id varchar(50) UNIQUE,
  sender varchar(10) NOT NULL,
  message varchar(500) NOT NULL
);
