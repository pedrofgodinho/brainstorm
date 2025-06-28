; c0 = 8
++++ ++++

; Sets cells to 00 00 0x48 0x68 0x20 0x08
[
    ; c1 = 4
    > ++++
    ; Set c2 = 8  c3 = 12  c4 = 12  c5 = 4
    [
        ; c2 add 2
        >++
        ; c3 add 3
        >+++
        ; c4 add 3
        >+++
        ; c5 add 1
        >+
        ; c1 sub 1
        <<<< -
    ]

    ; c2 add 1
    >+
    ; c3 add 1
    >+
    ; c4 sub 1
    >-
    ; c6 add 1
    >>+
    ; go to cell 0
    [<]

    ; c0 sub 1
    <-
]

; Print H
>>.
; Print e
>---.
; Print llo
+++++++..+++.
; Print space
>>.

; Print W
<-.
; Print o
<.
; Print rld
+++.------.--------.
; Print exclamation
>>+.
; Print newline
>++.