export default function ProgressBar({ progress }: { progress: number }) {
    let progressx100 = (progress * 100).toFixed(0)
    return (
        <div className="relative pt-1">
            <div className="flex mb-2 items-center justify-between">
                <div>
                    <span className="text-xs font-semibold inline-block py-1 px-2 uppercase rounded-full text-teal-600 bg-teal-200">
                        Progress
                    </span>
                </div>
                <div className="text-right">
                    <span className="text-xs font-semibold inline-block text-teal-600">
                        {progressx100}%
                    </span>
                </div>
            </div>
            <div className="flex mb-2 items-center justify-between">
                <div className="flex flex-col w-full">
                    <div className="w-full bg-gray-200 rounded-full h-2">
                        <div
                            style={{ width: `${progressx100}%` }}
                            className="shadow-none flex flex-col text-center whitespace-nowrap text-white justify-center bg-teal-500 rounded-full h-2"
                        ></div>
                    </div>
                </div>
            </div>
        </div>
    );
}