main:
        ASSIGN         |0              |               |i              
L1:
        GRE            |i              |5              |L0
        ASSIGN         |0              |               |j              
L3:
        GRE            |j              |5              |L2
        NEQ            |i              |j              |L4
        MUL            |j              |4              |T:0            
        MUL            |i              |20             |T:1            
        ADD            |T:0            |T:1            |T:2            
        ARRAYSTORE     |1              |T:2            |a              
        GOTO           |               |               |L5
L4:
        MUL            |j              |4              |T:3            
        MUL            |i              |20             |T:4            
        ADD            |T:3            |T:4            |T:5            
        ARRAYSTORE     |0              |T:5            |a              
L5:
        ADD            |j              |1              |j              
        GOTO           |               |               |L3
L2:
        ADD            |i              |1              |i              
        GOTO           |               |               |L1
L0:
        ASSIGN         |0              |               |i              
L7:
        GRE            |i              |5              |L6
        ASSIGN         |0              |               |j              
L9:
        GRE            |j              |5              |L8
        MUL            |j              |4              |T:6            
        MUL            |i              |20             |T:7            
        ADD            |T:6            |T:7            |T:8            
        ARRAYLOAD      |a              |T:8            |T:9            
        PARAM          |T:9            |               |
        CALL           |printi         |1              |
        ADD            |j              |1              |j              
        GOTO           |               |               |L9
L8:
        PARAM          |10             |               |
        CALL           |printc         |1              |
        ADD            |i              |1              |i              
        GOTO           |               |               |L7
L6:
----------------------------------------------------------