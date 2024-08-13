drop database oj;
create database oj;
use oj;

-- 用户表
create table user (
  id int primary key auto_increment, 
  avatar text,
  account char(32) unique,
  password char(64),
  join_time date,
  auth int
);