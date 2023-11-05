'use client'
import { invoke } from "@tauri-apps/api/tauri"
import { FormEvent, useEffect, useState } from "react"
import Input from "./input"

interface ConfigForm { [key: string]: string }

export default function ConfigForm() {
	let [config, setConfig] = useState<ConfigForm>({})

	useEffect(() => {
		let ignore = false
		const init = async () => {
			let new_config = await invoke("show_config") as ConfigForm
			if (!ignore) {
				setConfig(new_config)
			}
		}
		init()
		return () => { ignore = true }
	}, [])

	async function upgrade(e: FormEvent<HTMLFormElement>) {
		let form = new FormData(e.currentTarget)
		let json = Object.fromEntries(form)
		await invoke("upgrade_config", { json })
		setConfig(old => old)
	}

	return (
		<>
			<form className="flex flex-col place-items-center overflow-scroll" onSubmit={(e) => { e.preventDefault(); upgrade(e); }}>
				{Object.keys(config).map((key) => {
					return (
						<div key={key} className="grid grid-cols-3 mt-2 place-content-center">
							<label className="col-start-1 col-span-1 place-items-center mx-2 py-2"> {key} </label>
							<div className="col-start-2 col-span-2">
								<Input name={key} defaultValue={config[key]} />
							</div>
						</div>
					)
				})}
				<button type="submit"
					className="rounded-md bg-indigo-500 mt-4 w-fit px-3.5 py-2 text-sm font-semibold text-white shadow-sm 
                    hover:bg-indigo-400 
                    focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-500"
				> Upgrade Config </button>
			</form>
		</>
	)
}