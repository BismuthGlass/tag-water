create table if not exists entry (
	entry_id integer primary key,
	-- 1 = File
	-- 2 = Set
	entry_type integer		not null,
	parent_set integer 		default null,
	time_created integer	not null,
	time_updated integer	not null,

	-- File attributes
	ext text				,

	-- Set attributes
	cover integer			default null,
	foreign key(cover) references entry(entry_id)
);

create table if not exists set_file (
	set_id integer        	not null,
	file_id integer       	not null,
	position integer        not null default 1,
	primary key(set_id, file_id),
	foreign key(set_id) references entry(entry_id),
	foreign key(file_id) references entry(entry_id)
);

create table if not exists tag (
	tag_id integer primary key,
	name text           	not null unique,
	description text      	not null default "",
	category integer       	not null default 1,
	time_created integer   	not null,
	time_updated integer    	not null,
	foreign key(category) references tag_category(tcat_id)
);

create table if not exists tag_category (
	tcat_id integer primary key,
	name text				not null unique,
	description text		not null default "",
	time_created integer    not null,
	time_updated integer		not null
);

create table if not exists entry_tag (
	entry_id integer       	not null,
	tag_id integer        	not null,
	primary key(entry_id, tag_id),
	foreign key(entry_id) references entry(entry_id),
	foreign key(tag_id) references tag(tag_id)
);

create table if not exists upload_file (
	id integer primary key,
	ext text               not null default "",
	title text             not null
);

-- Default system values
insert into tag_category (name, time_created, time_updated) values 
	("default", 0, 0),
	("system", 0, 0);
