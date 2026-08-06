tests = open("work/w-fnbyte/newtests.rs", encoding="utf-8").read()
p = "crates/c2-harness/src/gap/tests.rs"
with open(p, "a", encoding="utf-8") as f:
    f.write(tests)
print("appended", len(tests), "bytes")
