import ConfigForm from "@/components/configForm";
import HomeBtn from "@/components/homeBtn";

export default function ConfigPage() {
    return (
        <div className="flex flex-col place-items-center gap-4">
            <ConfigForm />
            <HomeBtn />
        </div>
    )
}