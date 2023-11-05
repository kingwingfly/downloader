import { InputHTMLAttributes } from "react";

interface InputAttr extends InputHTMLAttributes<HTMLInputElement> {
    name?: string,
    type?: string
}

export default function Input(props: InputAttr) {
    return (
        <input required {...props}
            className="min-w-0 rounded-md border-0 w-fit
            bg-white/5 px-3.5 py-2 mx-1 shadow-sm ring-1 ring-inset ring-blue
            focus:ring-2 focus:ring-indigo-500 sm:text-sm sm:leading-6"
            placeholder={props.placeholder ? props.placeholder : `Enter the ${props.name}`}
        />
    )
}