-- Add migration script here
create table dimensions (
    id serial primary key,
    name varchar(255) not null
);

insert into dimensions (name) values ('The Overworld'), ('The Nether'), ('The End');

create table waypoints (
    id serial primary key,
    name varchar(255) not null,
    x int4 not null,
    y int4 not null,
    z int4 not null,
    dimension integer references dimensions(id),
    completed bool
);
