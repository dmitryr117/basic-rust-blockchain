
- Something is wrong with transaction hashing. possibly invalid inputs.

Output map transaction order issue.

In this in `p2p_task.rs`.

```
match topic_enum {
	TopicEnum::Blockchain => {
    // chain replacement
		if let Ok(new_chain) = Blockchain::from_bytes(&message.data) {
```

Because from_bytes is impossible to deal with because hash_map can be inconsistent.


- Need to encode - decode - hash - unhash. for any data that has to be hashed to avoid issues such as
HashMap out of order problems.