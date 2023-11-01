'use client'

import HomeBtn from "@/components/homeBtn"
import TaskCard, { CardInfo } from "@/components/taskCard";
import { invoke } from "@tauri-apps/api"
import { useCallback, useState } from "react";

export default function TaskList() {
    let [infos, setInfos] = useState<CardInfo[]>([])

    const run = useCallback(async () => {
        if (infos.length !== 0) {
            await new Promise(resolve => setTimeout(resolve, 1000))
        }
        let new_infos = await invoke("progress") as CardInfo[]
        if (new_infos.length !== 0) {
            console.log(1)
            setInfos(new_infos)
        }
    }, [infos])

    run()


    return (
        <>
            <div className="flex-col max-h-screen overflow-auto p-12">{infos.map(info => <TaskCard key={info[3]} info={info} />)}</div>
            <HomeBtn />
        </>
    )
}