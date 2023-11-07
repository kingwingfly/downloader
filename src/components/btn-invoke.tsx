'use client'

import { ButtonHTMLAttributes } from "react"
import { invoke } from "@tauri-apps/api/tauri";

interface invokeAttri extends ButtonHTMLAttributes<HTMLButtonElement> {
    func: string,
    params?: { [key: string]: any }
    desc?: string
}

// No return, just call
export default function BtnInvoke(props: invokeAttri) {
    return (
        <button className={`
        rounded-md ${props.disabled ? "bg-red-500" : "bg-indigo-500"} 
        mx-4 px-3.5 py-2 text-sm font-semibold text-white shadow-sm 
        hover:${props.disabled ? "bg-red-400" : "bg-indigo-400"} 
        focus-visible:outline focus-visible:outline-2 
        focus-visible:outline-offset-2 focus-visible:outline-indigo-500
        `}
            onClick={async () => { await invoke(props.func, props.params) }} {...props}> {props.desc} </button>
    )
}