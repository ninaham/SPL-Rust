gcd:
L1:
        EQU            |a              |b              |L0
        GRE            |a              |b              |L2
        SUB            |b              |a              |b              
        GOTO           |               |               |L3
L2:
        SUB            |a              |b              |a              
L3:
        GOTO           |               |               |L1
L0:
        ASSIGN         |a              |               |r              
----------------------------------------------------------

main:
        ASSIGN         |10164          |               |a              
        ASSIGN         |2646           |               |b              
        PARAM          |a              |               |
        PARAM          |b              |               |
        PARAM          |c              |               |
        CALL           |gcd            |3              |
        PARAM          |a              |               |
        CALL           |printi         |1              |
        PARAM          |32             |               |
        CALL           |printc         |1              |
        PARAM          |b              |               |
        CALL           |printi         |1              |
        PARAM          |32             |               |
        CALL           |printc         |1              |
        PARAM          |c              |               |
        CALL           |printi         |1              |
        PARAM          |10             |               |
        CALL           |printc         |1              |
----------------------------------------------------------