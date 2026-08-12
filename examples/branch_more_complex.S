.text
    .global _start

_start:
	mov	r0, #0
	bl	foo

	mov	r7, #1
	svc	#0

foo:
	mov	r0, #42
	mov	r7, #1
	svc	#0
