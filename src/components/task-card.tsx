'use client'

import { stat } from "fs"
import BtnInvoke from "./btn-invoke"
import ProgressBar from "./progress"

// title finished total uuid state
export type CardInfo = [string, number, number, string, string]

export default function TaskCard({ info }: { info: CardInfo }) {
    // `>>20` to Mb then `<<1 + <<3` to x10
    // so `(>>19 + >>17) / 10` keeps to one decimal place
    let progress = info[1] / info[2]
    let finished = ((info[1] >> 19) + (info[1] >> 17)) / 10
    let total = ((info[2] >> 19) + (info[2] >> 17)) / 10
    return (
        <div className="task_card p-4 border rounded-lg shadow-md">
            <h1 className="text-xl font-bold mb-2">{info[0]}</h1>
            <div className="text-gray-700">{`${finished}/${total} Mb`}</div>
            <ProgressBar progress={progress} state={info[4]} />
            <div className="btns pt-4 flex justify-center">
                {buttons(info[4], info[3])}
            </div>
        </div>
    )
}

function buttons(state: string, id: string) {
    return (
        <>
            {
                state == "downloading" && <BtnInvoke func="pause" params={{ id }} desc="Pause" />
            }
            {
                state == "pausing" && <BtnInvoke func="" desc="Pausing..." disabled />
            }
            {
                state == "paused" && <BtnInvoke func="continue_" params={{ id }} desc="Continue" />
            }
            {
                (state == "cancelled" || state == "finished") || <BtnInvoke func="cancel" params={{ id }} desc="Cancel" />
            }
            <BtnInvoke func="remove" params={{ id }} desc="Remove" />
        </>)
}