drop database oj;
create database oj;
use oj;

-- 用户表
create table user (
  id int primary key auto_increment, 
  avatar text,
  account char(32) unique not null,
  password char(64) not null,
  join_time date,
  auth int not null
);