import Image from "next/image"
import Link from "next/link"

export default function ConfigIcon() {
    return (
        <>
            <Link href={"/config"} className="h-fit w-fit" scroll={false}>
                <Image className="dark:invert"
                    src={"/config.svg"} alt={"config.svg"} width={70} height={70} />
            </Link>
        </>
    )
}