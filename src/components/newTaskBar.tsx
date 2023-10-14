'use client'
import { FormEvent } from "react";
import Input from "./input";

export default function NewTaskBar() {
    const onsubmit = async (e: FormEvent<HTMLFormElement>) => {
        e.preventDefault()
        let form = new FormData(e.currentTarget);
        let url = form.get('url')
        console.log(url)
    }
    return (
        <>
            <form onSubmit={(e) => onsubmit(e)} className="flex">
                <label htmlFor="url"></label>
                <Input name="url" type="url" />
                <button type="submit"
                    className="rounded-md bg-indigo-500 px-3.5 py-2. ml-4 text-sm font-semibold text-white shadow-sm hover:bg-indigo-400 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-500"
                >
                    Go
                </button>
            </form>
        </>
    )
}
