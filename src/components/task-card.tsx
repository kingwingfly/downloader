'use client'

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
                {
                    info[4] == "pausing" ? <BtnInvoke func={""} desc="Pausing" disabled={true} /> :
                        info[4] == "paused" ?
                            <BtnInvoke func={"continue_"} params={{ id: info[3] }} desc="Continue" />
                            :
                            <BtnInvoke func={"pause"} params={{ id: info[3] }} desc="Pause" />
                }
                <BtnInvoke func={"cancel"} params={{ id: info[3] }} desc="Cancel" />
                <BtnInvoke func={"remove"} params={{ id: info[3] }} desc="Remove" />
            </div>
        </div>
    )
}