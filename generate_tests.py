import token
from tokenize import tokenize
from pathlib import Path
from argparse import ArgumentParser


def token_type_from_python_to_rust(typefield):
    match typefield:
        case token.ENCODING:
            return "TType::Encoding"
        case token.STRING:
            return "TType::String"
        case token.NAME:
            return "TType:Name"
        case token.OP:
            return "TType:Op"
        case token.NEWLINE:
            return "TType::Newline"
        case token.NUMBER:
            return "TType::Number"
        case token.INDENT:
            return "TType::Indent"
        case token.DEDENT:
            return "TType::Dedent"
        case token.ENDMARKER:
            return "TType::Endmarker"
        case token.NL:
            return "TType::NL"
        
        case default:
            return f"Not handled yet {typefield}"



def walk_workingpath(work_path:Path):
    for element in work_path.glob("*.py"):
        if element.is_file():

            with element.open("rb") as my_file:
                print(f"Processing: {element}")
                print("="*80)
                try:
                    tokens = tokenize(my_file.readline)
                    for idx, token in enumerate(tokens):
                        print(token)
                        print(f"test_token!(tokens[{idx}], {token_type_from_python_to_rust(token.type)}, \"{token.string}\" );")
                except Exception as exc:
                    print("Failed to tokenize because {exc}")

                print("Finished\n")




def main():
    parser = ArgumentParser()
    parser.add_argument("work_path", help="Path filled with python files to be tokenized.", type=Path)

    args = parser.parse_args()

    walk_workingpath(args.work_path)



if __name__ == '__main__':
    main()

