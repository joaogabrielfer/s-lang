push 20 30
add
dup
eq 50 if {
	into var a
} else {
	push 30
	sub
	into var a
}

dup a
eq a if {
	push a
	pop
}

true if {
	push 3
	pop
}
