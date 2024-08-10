drop database oj;
create database oj;
use oj;
create table user (
  id int primary key auto_increment, 
  avatar text,
  account char(32),
  password char(64),
  join_time date,
  authority int
);

