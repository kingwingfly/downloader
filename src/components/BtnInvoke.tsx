'use client'

import { invoke } from "@tauri-apps/api";

interface invoke {
    func: string,
    params?: { [key: string]: any }
    desc?: string
}

// No return, just call
export default function BtnInvoke({ func, params, desc }: invoke) {
    return (
        <button className="
        rounded-md bg-indigo-500 mx-4 px-3.5 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-indigo-400 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-500
        "
            onClick={async () => { await invoke(func, params) }}> {desc} </button>
    )
}