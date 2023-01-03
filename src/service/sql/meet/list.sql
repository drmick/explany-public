select m.id as "id!",
       m.title as "title!",
       (extract(epoch from lower(m.range)) * 1000)::bigint as "from!",
       (extract(epoch from upper(m.range)) * 1000)::bigint as "to!",
       m.status_id as "status_id!",
       ms.title_ru as "status_title!",
       sp.last_name as "spec_last_name!",
       sp.first_name as "spec_first_name!",
       sp.avatar_thumb_url as "spec_avatar_thumb_url",
       m.room as "room!",
       u.first_name || ' ' ||u.last_name  as "user_first_name!",
       s.name as "service_name!",
        (select coalesce(array_agg(mf.status_id), '{}') from meet_status_flow  mf
       where mf.parent_status_id = m.status_id and mf.role_id = $2 ) as "actions!"

from meets m
join meet_statuses ms
on ms.id = m.status_id
join spec sp
on sp.id = m.spec_id
join services s
on m.service_id = s.id
join users u on m.user_id = u.id
where m.id = coalesce($3, m.id)
and (m.user_id = $1 or m.spec_id = $1)
order by case when m.status_id = 'Canceled' then  -1 when m.status_id = 'Finished' then 0 else 1 end desc,
    lower(m.range)
