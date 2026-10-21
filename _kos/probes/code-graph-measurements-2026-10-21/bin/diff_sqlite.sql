.timer on
SELECT 'added', count(*) FROM (SELECT e,a,v FROM facts WHERE tx=2 EXCEPT SELECT e,a,v FROM facts WHERE tx=1);
SELECT 'removed', count(*) FROM (SELECT e,a,v FROM facts WHERE tx=1 EXCEPT SELECT e,a,v FROM facts WHERE tx=2);
SELECT 'added_by_a', a, count(*) FROM (SELECT e,a,v FROM facts WHERE tx=2 EXCEPT SELECT e,a,v FROM facts WHERE tx=1) GROUP BY a;
SELECT 'removed_by_a', a, count(*) FROM (SELECT e,a,v FROM facts WHERE tx=1 EXCEPT SELECT e,a,v FROM facts WHERE tx=2) GROUP BY a;
