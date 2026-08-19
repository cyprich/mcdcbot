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
    dimension integer references dimensions(id),  -- TODO: shouldnt this be not null?
    completed bool
);

create table pending_waypoints (
    id serial primary key,
    action varchar(16) not null,  -- add, edit, delete
    author_name varchar(255) not null,
    author_id varchar(64) not null,  -- TODO: what is the actual length? could be just char

    waypoint_id integer references waypoints(id),
    name varchar(255),
    x int4,
    y int4,
    z int4,
    dimension integer,
    -- cant have `Option<Option<bool>>` in postgres
    completed_changed bool,
    completed_value bool
);

select * from mc.waypoints;
