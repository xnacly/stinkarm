.text
	.global _start
_start:
	mov	r0, #0
	b	1f
	mov	r0, #1		@ must NOT execute
1:
	mov	r7, #1
	svc	#0

