'use client'

import TaskCard, { CardInfo } from "@/components/taskCard";
import { invoke } from "@tauri-apps/api"
import { useEffect, useState } from "react";

export default function TaskList() {
    let [infos, setInfos] = useState<{ infos: CardInfo[], sleep: number }>({ infos: [], sleep: 0 })
    // TODO rerender only when infos have changed

    useEffect(() => {
        let ignore = false
        const run = async () => {
            await new Promise(resolve => setTimeout(resolve, infos.sleep))
            let new_infos = await invoke("progress") as CardInfo[]
            if (!ignore) {
                console.log(new_infos)
                setInfos({ infos: new_infos, sleep: 1000 })
            }
        }
        run()
        return () => { ignore = true; }
    }, [infos]
    )


    return (
        <>{
            infos.infos.length !== 0
                ?
                <div className="flex-col max-h-screen overflow-auto">{infos.infos.map(info => <TaskCard key={info[3]} info={info} />)}</div>
                :
                <div>Empty</div>
        }</>
    )
}