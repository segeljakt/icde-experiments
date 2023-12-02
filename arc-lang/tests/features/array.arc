val a = [1,2,3,4];

val v0 = a.array_get(0);
val v1 = a.array_get(1);

print(i32_to_string(v0 + v1));

val a2 = a.array_set(0, 10);

val v2 = a2.array_get(0);
val v3 = a2.array_get(1);
