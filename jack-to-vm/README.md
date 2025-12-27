# Jack to VM compiler

This crate operates on `.jack` files and produces intermediate representation vm programs:
- Jack classes are tokenized and parsed into derivation trees
- the derivation tree is traversed until all language statements and expressions have been compiled down to `.vm` programs

To provide a simple example, the following Jack class

```
class Main {
     function void main() {
         return;
     }
}
```

... should evaluate to the following derivation tree:

```xml
<class>
    <keyword>class</keyword>
    <identifier>Main</identifier>
    <symbol>{</symbol>
    <subroutineDec>
        <keyword>function</keyword>
        <keyword>void</keyword>
        <identifier>main</identifier>
        <symbol>(</symbol>
        <symbol>)</symbol>
        <symbol>{</symbol>
            <subroutineBody>
                <statements>
                    <returnStatement>
                        <keyword>return</keyword>
                        <symbol>;</symbol>
                    </returnStatement>
                </statements>
            </subroutineBody>
        <symbol>}</symbol>
    </subroutineDec>
    <symbol>}</symbol>
</class>
```

... which should then be compiled to the following intermediate VM program:

```
function Main.main 0
push constant 0
return
```
