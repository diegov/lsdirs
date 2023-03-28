drop table freq_path;

create table freq_path (
  id integer primary key autoincrement,
  canonical_path text not null,
  last_seen integer null,
  seen_count real null
);

create unique index unq_freq_path_canonical_path on freq_path (canonical_path);
create index idx_freq_path_last_seen on freq_path (last_seen);
