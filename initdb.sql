drop database oj;
create database oj;
use oj;

-- 用户表
create table user (
  uid int primary key auto_increment, 
  avatar text,
  account char(32) unique not null,
  password char(64) not null,
  join_time date,
  auth int not null
);

INSERT INTO user (avatar, account, password, join_time, auth)
VALUES ('null', 'admin', 'jzm19260817', CURDATE(), 30);

-- 题目表
create table problem (
  pid int primary key auto_increment, 
  title char(64) not null,
  description text,
  judge_num int not null default 0,
  time_limit time(3) not null default "00:00:01",
  memory_limit int not null default 128000,
  uid int,
  foreign key(uid) references user(uid) on delete set null
);