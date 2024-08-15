drop database oj;
create database oj;
use oj;

-- 用户表
create table user (
  uid int primary key auto_increment, 
  avatar text,
  account char(32) unique,
  password char(64),
  join_time date,
  auth int
);

INSERT INTO user (avatar, account, password, join_time, auth)
VALUES ('null', 'admin', 'jzm19260817', CURDATE(), 30);

-- 题目表
create table problem (
  pid int primary key auto_increment, 
  title char(64) not null,
  description text,
  judge_num int,
  time_limit float,
  memory_limit int,
  uid int,
  foreign key(uid) references user(uid) on delete set null
);